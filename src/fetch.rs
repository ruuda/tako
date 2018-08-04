// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Contains the main fetching logic (downloading manifests and images).

use std::fs;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::os::unix;
use std::path::Path;

use sodiumoxide::crypto::hash::sha256;

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

    // TODO: Put a limit on the size of the manifest, to protect against
    // malicious mirrors serving large manifests that fill up the disk.
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

fn fetch_image(
    uri: &str,
    target_fname: &Path,
    len: u64,
    digest: &sha256::Digest,
    curl_handle: &mut curl::Handle
) -> Result<()> {
    // Download to store/<hexdigest>.new. Then later rename the file to its
    // final path. This ensures that when the program crashes or is killed mid-
    // download, next time we will start the download again immediately. Also,
    // this guarantees that the files in the store that don't have a ".new"
    // suffix are valid (if nothing external modifies them).
    let tmp_fname = target_fname.with_extension("new");

    // In case of error, delete the temp file.
    let guard = util::FileGuard::new(&tmp_fname);

    let mut ctx = sha256::State::new();
    {
        let ctx_ref = &mut ctx;
        let mut f = BufWriter::new(fs::File::create(&tmp_fname)?);
        let mut bytes_written = 0;
        curl_handle.download_err(uri, |chunk| {
            if bytes_written + chunk.len() as u64 > len {
                Err(Error::InvalidSize)
            } else {
                bytes_written += chunk.len() as u64;
                ctx_ref.update(chunk);
                f.write_all(chunk)?;
                Ok(())
            }
        })?;

        if bytes_written != len {
            return Err(Error::InvalidSize)
        }
    }
    let actual_digest = ctx.finalize();

    let is_digest_valid = actual_digest == *digest;

    if !is_digest_valid {
        return Err(Error::InvalidDigest)
    }

    // The store should be immutable, make the file readonly. Then move it into
    // its final place.
    guard.move_readonly(&target_fname)?;

    Ok(())
}

/// Create the symlink to the target path `store/<hexdigest>`.
///
/// This is a no-op if the symlink exists and points to the target path already.
fn update_symlink<P: AsRef<Path>>(config: &Config, target_path: P) -> io::Result<()> {
    let mut sympath = config.destination.clone();
    sympath.push("latest");

    match sympath.read_link() {
        Ok(ref points_at) if points_at == target_path.as_ref() => return Ok(()),
        // Other cases are nonexisting symlink, or symlink pointing at
        // something else than the target. In both cases we create (overwrite)
        // the symlink.
        _ => unix::fs::symlink(target_path.as_ref(), sympath)
    }
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
    let store_path = &uri[prefix_len..];

    println!("Fetching {} from {} ...", candidate.version.as_str(), uri);

    // The target filename is store/<hexdigest> in the configured
    // destination directory.
    let mut target_fname = config.destination.clone();
    target_fname.push(store_path);

    // Create the store directory inside the target directory, if it does not
    // exist already. Do not create any of the parent dirs, this is the
    // responsibility of the user. The unwrap is safe here; by construction the
    // path has at least two components.
    let store_dirname = target_fname.parent().unwrap();
    if !store_dirname.is_dir() {
        fs::create_dir(store_dirname)?;
    }

    if target_fname.is_file() {
        // If the target file exists in the store already, don't download it
        // again, but do verify its integrity. If damaged, delete the file from
        // the store, such that on the next run we will download it again, and
        // also to prevent the damaged (or tampered with) file from being used.
        if util::sha256sum(&target_fname)? != candidate.digest {
            let _ = fs::remove_file(&target_fname);
            // TODO: Also delete the symlink if it happened to point at the
            // corrupted file?
            return Err(Error::InvalidDigest)
        }
    } else {
        // If the file was not in the store, download it. This performs an on
        // the fly integrity check.
        fetch_image(&uri, &target_fname, candidate.len, &candidate.digest, &mut curl_handle)?;
    }

    update_symlink(&config, &store_path)?;

    Ok(())
}
