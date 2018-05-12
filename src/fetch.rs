// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Contains the main fetching logic (downloading manifests and images).

use std::fs;
use std::io;
use std::io::{BufRead, BufWriter, Read, Write};

use ring::digest;

use config::Config;
use curl;
use error::{Error, Result};
use manifest;
use manifest::Manifest;
use util;

fn load_config(config_fname: &str) -> Result<Config> {
    let f = fs::File::open(config_fname)?;
    let buf_reader = io::BufReader::new(f);
    let lines: io::Result<Vec<String>> = buf_reader.lines().collect();
    Config::parse(lines?.iter())
}

/// Load a locally stored manifest.
fn load_local_manifest(config: &Config) -> Result<Option<Manifest>> {
    // Open the current manifest. If it does not exist that is not an error.
    let mut path = config.destination.clone();
    path.push("manifest");
    let f = match fs::File::open(path) {
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        other => other?,
    };

    let mut buf_reader = io::BufReader::new(f);
    let mut manifest_bytes = Vec::new();
    buf_reader.read_to_end(&mut manifest_bytes)?;

    Ok(Some(Manifest::parse(&manifest_bytes[..], &config.public_key)?))
}

/// Store a manifest locally. Writes first and then swaps the file.
fn store_local_manifest(config: &Config, bytes: &[u8]) -> Result<()> {
    let mut path_tmp = config.destination.clone();
    let mut path_final = config.destination.clone();
    path_tmp.push("manifest.new");
    path_final.push("manifest");

    // First write the entire manifest to a new file.
    let f = fs::File::create(&path_tmp)?;
    let mut buf_writer = io::BufWriter::new(f);
    buf_writer.write_all(bytes)?;

    // Then rename it over the old manifest.
    fs::rename(path_tmp, path_final)?;

    Ok(())
}

/// Fetch the remote manifest, store it locally if it is valid, and return it.
pub fn fetch_manifest(config: &Config, curl_handle: &mut curl::Handle) -> Result<Manifest> {
    // TODO: If we fail to load this manifest, it is not clear to the user
    // that this is about the local manifest, rather than the remote one. We
    // should extend the error type to include this info.
    // TODO: In the case of a key rotation, after updating the key in the
    // config, we would no longer be able to load the currently stored manifest.
    // How to deal with that? Allow multiple public keys in the config?
    let local_manifest = Manifest::load_local(&config.destination, &config.public_key)?;

    let mut uri = config.origin.to_string();
    if !uri.ends_with("/") { uri.push('/'); }
    uri.push_str("manifest");

    let mut manifest_bytes = Vec::new();
    curl_handle.download(&uri, |chunk| manifest_bytes.extend_from_slice(chunk))?;

    let remote_manifest = Manifest::parse(&manifest_bytes[..], &config.public_key)?;

    // If there was a local manifest already, it must be a subset of the remote
    // one. Otherwise, if we overwrite the local manifest, that would remove
    // entries, and those entries might exist on disk -- one of them might be
    // the image currently in use. If we would erase that from the manifest,
    // then we would no longer know what that image is. So bail out.
    if Some(false) == local_manifest.map(|m| m.is_subset_of(&remote_manifest)) {
        let msg = "The remote manifest is not a superset of the local manifest. Rejecting remote manifest.";
        return Err(Error::OperationError(msg))
    }

    // Store the manifest locally before we continue. It doesn't hurt to have
    // more entries in there even if we don't have the images yet. But on the
    // other hand, if an image exists locally, it had better be in the manifest.
    manifest::store_local(&config.destination, &manifest_bytes[..])?;

    Ok(remote_manifest)
}

/// Check for, download, and apply updates as given in the config.
pub fn fetch(config_fname: &str) -> Result<()> {
    let config = load_config(config_fname)?;
    println!("config: {:?}", config);

    let mut curl_handle = curl::Handle::new();

    let manifest = fetch_manifest(&config, &mut curl_handle)?;

    let (lower, upper) = config.version.pattern_to_bounds();
    let candidate = manifest.latest_compatible_entry(&lower, &upper).ok_or(Error::NoCandidate)?;

    let mut uri = config.origin.to_string();
    if !uri.ends_with("/") { uri.push('/'); }
    let prefix_len = uri.len();
    uri.push_str("store/");
    util::append_hex(&mut uri, candidate.digest.as_ref());

    println!("Fetching {} from {} ...", candidate.version.as_str(), uri);

    // The target filename is store/<hexdigest> in the configured
    // destination directory.
    let mut target_fname = config.destination.clone();
    target_fname.push(&uri[prefix_len..]);

    // TODO: Maybe write to temp file, rename afterwards, after hash
    // verification?

    // Create the store directory inside the target directory, if it does not
    // exist already. Do not create any of the parent dirs, this is the
    // responsibility of the user. The unwrap is safe here; by construction the
    // path has at least two components.
    let store_dirname = target_fname.parent().unwrap();
    if !store_dirname.is_dir() {
        fs::create_dir(store_dirname)?;
    }

    // TODO: Check if file exists before downloading.

    // Download to store/<hexdigest>.new. Then later rename the file to its
    // final path. This ensures that when the program crashes or is killed mid-
    // download, next time we will start again immediately. Also, this
    // guarantees that the files in the store that don't have a ".new" suffix
    // are valid (if nothing external modifies them).
    let tmp_fname = target_fname.with_extension("new");

    let mut ctx = digest::Context::new(&digest::SHA256);
    {
        let ctx_ref = &mut ctx;
        let mut f = BufWriter::new(fs::File::create(&tmp_fname)?);
        curl_handle.download_io(&uri, |chunk| {
            ctx_ref.update(chunk);
            f.write_all(chunk)
        })?;
    }
    let actual_digest = ctx.finish();

    // The comparison is not constant time, but that is not an issue here; a
    // digest cannot be bruteforced byte by byte until it matches.
    let is_digest_valid = actual_digest.as_ref() == candidate.digest.as_ref();

    if !is_digest_valid {
        // Remove the temp file. It is corrupted somehow, it is no use. This is
        // an operation that may fail, but we are already in a failure mode, and
        // the "invalid digest" error is arguably more informative than an IO
        // failure, so return that one and ignore any IO failures. In other
        // words: report the original error, not any error that happens during
        // error handling.
        let _ = fs::remove_file(&tmp_fname);
        return Err(Error::InvalidDigest)
    }

    // The store should be immutable, make the file readonly. Then move it into
    // its final place.
    let mut perms = fs::metadata(&tmp_fname)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&tmp_fname, perms)?;
    fs::rename(tmp_fname, &target_fname)?;

    Ok(())
}
