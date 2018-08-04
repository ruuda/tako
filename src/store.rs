// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Contains the main store logic.

use std::fs;
use std::io::Read;
use std::path::PathBuf;

use base64;
use ring::signature::Ed25519KeyPair;
use sodiumoxide::crypto::sign::ed25519;
use untrusted::Input;

use cli::Store;
use error::{Error, Result};
use manifest;
use manifest::{Entry, Manifest};
use util;

pub fn store(store: Store) -> Result<()> {
    let secret_keypair_seed_base64 = match (store.secret_key, store.secret_key_path) {
        (Some(k), _) => k,
        (None, Some(p)) => {
            let mut s = String::new();
            // Don't use a BufReader here, that would be pointless: we are
            // already reading into a (string) buffer.
            let mut f = fs::File::open(p)?;
            f.read_to_string(&mut s)?;
            // The base64-encoded seed of the keypair is 43 bytes long, plus 8
            // bytes of "SECRET:" to distinguish the seed from the public key.
            // There might be a trailing newline at the end of the file that we
            // discard here. There might also be junk, then we find out later
            // when parsing the base64.
            s.truncate(43 + 7);
            s
        }
        (None, None) => unreachable!("Should have been validated elsewhere."),
    };

    // The keypair seed is the same size as the public key, so to distinguish,
    // we prefix the (secret) seed with "SECRET:", and if it's not there, reject
    // the seed.
    let err = Err(Error::InvalidSecretKeyData);
    match &secret_keypair_seed_base64[..7] {
        "SECRET:" => { /* Ok, as expected. */ }
        _ => return err,
    }

    let err = Err(Error::InvalidSecretKeyData);
    let secret_keypair_seed_bytes = base64::decode(&secret_keypair_seed_base64[7..]).or(err)?;

    let err = Error::InvalidSecretKeyData;
    let secret_keypair_seed = ed25519::Seed::from_slice(&secret_keypair_seed_bytes).ok_or(err)?;

    let (public_key, secret_key) = ed25519::keypair_from_seed(&secret_keypair_seed);

    let mut manifest = match Manifest::load_local(&store.output_path, &public_key)? {
        Some(m) => m,
        None => Manifest::new(),
    };

    let mut store_dir = PathBuf::from(&store.output_path);
    store_dir.push("store");

    // The server directory must exist, but we can create the store directory
    // inside there, in case we are constructing a completely new
    // store/manifest.
    if !store_dir.is_dir() {
        fs::create_dir(&store_dir)?;
    }

    let digest = util::sha256sum(&store.image_path)?;
    let mut digest_hex = String::new();
    util::append_hex(&mut digest_hex, digest.as_ref());

    let mut target_fname = store_dir;
    target_fname.push(&digest_hex);

    // Copy the image into the store under its content-based name. If the target
    // exists, verify the checksum instead.
    if target_fname.is_file() {
        // TODO: Verify SHA256.
    } else {
        fs::copy(&store.image_path, &target_fname)?;
    }

    // The store should be immutable, make the file readonly.
    let metadata = fs::metadata(&target_fname)?;
    let mut perms = metadata.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&target_fname, perms)?;

    println!("{} -> {}", store.version.as_str(), digest_hex);

    // Add the new entry to the manifest.
    let entry = Entry {
        version: store.version,
        len: metadata.len(),
        digest: digest,
    };
    manifest.insert(entry)?;

    // And finally store the new manifest. Write to a temporary file, then swap
    // it into place.
    let manifest_string = manifest.serialize(&secret_key);
    manifest::store_local(&store.output_path, manifest_string.as_bytes())?;

    Ok(())
}
