// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Contains the main store logic.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use base64;
use ring;
use ring::signature::Ed25519KeyPair;
use untrusted::Input;

use cli::Store;
use config::PublicKey;
use error::{Error, Result};
use manifest;
use manifest::{Entry, Manifest, Sha256};
use util;

pub fn sha256sum(path: &Path) -> Result<Sha256> {
    // TODO: Use Filebuffer mmap for this once I have an internet connection
    // again. Cargo does not let me add a dependency without an internet
    // connection. Even if I specify a dependency by local path, this does not
    // work, unfortunately.
    let mut bytes = Vec::new();
    let mut f = fs::File::open(path)?;
    f.read_to_end(&mut bytes)?;
    let sha256_bytes = ring::digest::digest(&ring::digest::SHA256, &bytes);
    Ok(Sha256::copy_from_slice(sha256_bytes.as_ref()))
}

pub fn store(store: Store) -> Result<()> {
    let secret_key_base64 = match (store.secret_key, store.secret_key_path) {
        (Some(k), _) => k,
        (None, Some(p)) => {
            let mut s = String::new();
            // Don't use a BufReader here, that would be pointless: we are
            // already reading into a (string) buffer.
            let mut f = fs::File::open(p)?;
            f.read_to_string(&mut s)?;
            // The base64-encoded secret key is 116 bytes long. There might be
            // a trailing newline at the end of the file that we discard here.
            // There might also be junk, then we find out later when parsing the
            // base64.
            s.truncate(116);
            s
        }
        (None, None) => unreachable!("Should have been validated elsewhere."),
    };

    let err = Err(Error::InvalidSecretKeyData);
    let secret_key_bytes = base64::decode(&secret_key_base64).or(err)?;

    let err = Err(Error::InvalidSecretKeyData);
    let key_pair = Ed25519KeyPair::from_pkcs8(Input::from(&secret_key_bytes)).or(err)?;
    let public_key = PublicKey::from_pair(&key_pair);

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

    let digest = sha256sum(&store.image_path)?;
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
    let mut perms = fs::metadata(&target_fname)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&target_fname, perms)?;

    println!("{} -> {}", store.version, digest_hex);

    // Add the new entry to the manifest.
    let entry = Entry {
        version: store.version,
        digest: digest,
    };
    manifest.entries.push(entry);

    // TODO: Sort and deduplicate. Verify that versions do not occur twice.

    // And finally store the new manifest. Write to a temporary file, then swap
    // it into place.
    let manifest_string = manifest.serialize(&key_pair);
    manifest::store_local(&store.output_path, manifest_string.as_bytes())?;

    Ok(())
}
