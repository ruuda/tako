// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Contains the main fetching logic (downloading manifests and images).

use std::fs;
use std::io;
use std::io::{BufRead, Read, Write};

use config::Config;
use curl;
use error::{Error, Result};
use manifest;
use manifest::Manifest;

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

/// Check for, download, and apply updates as given in the config.
pub fn fetch(config_fname: &str) -> Result<()> {
    let config = load_config(config_fname)?;
    println!("config: {:?}", config);

    // TODO: If we fail to load this manifest, it is not clear to the user
    // that this is about the local manifest, rather than the remote one. We
    // should extend the error type to include this info.
    // TODO: In the case of a key rotation, after updating the key in the
    // config, we would no longer be able to load the currently stored manifest.
    // How to deal with that? Allow multiple public keys in the config?
    let local_manifest = Manifest::load_local(&config.destination, &config.public_key)?;

    let mut curl_handle = curl::Handle::new();

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

    Ok(())
}
