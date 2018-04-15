// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Contains the main fetching logic (downloading manifests and images).

use std::fs;
use std::io;
use std::io::{BufRead};

use config::Config;
use curl;
use error::Result;
use manifest::Manifest;

fn load_config(config_fname: &str) -> Result<Config> {
    let f = fs::File::open(config_fname)?;
    let buf_reader = io::BufReader::new(f);
    let mut lines = Vec::new();
    for line in buf_reader.lines() {
        lines.push(line?);
    }
    Config::parse(lines.iter())
}

/// Check for, download, and apply updates as given in the config.
pub fn fetch(config_fname: &str) -> Result<()> {
    let config = load_config(config_fname)?;
    println!("config: {:?}", config);

    let mut curl_handle = curl::Handle::new();

    let mut uri = config.origin.to_string();
    if !uri.ends_with("/") { uri.push('/'); }
    uri.push_str("manifest");

    let mut manifest_bytes = Vec::new();
    curl_handle.download(&uri, |chunk| manifest_bytes.extend_from_slice(chunk))?;

    let manifest = Manifest::parse(&manifest_bytes[..])?;
    // TODO: Verify signature.

    for entry in &manifest.entries {
        println!("entry: {:?}", entry);
    }

    Ok(())
}
