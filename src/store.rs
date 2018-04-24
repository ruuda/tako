// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Contains the main store logic.

use std::fs;
use std::io::Read;

use base64;
use ring::signature::Ed25519KeyPair;
use untrusted::Input;

use cli::Store;
use config::PublicKey;
use error::{Error, Result};
use manifest::Manifest;

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

    let current_manifest = match Manifest::load_local(&store.output_path, &public_key)? {
        Some(m) => m,
        None => Manifest::new(),
    };

    unimplemented!("TODO: Read old manifest, append, construct write new.");
}
