// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Utilities for formatting, parsing, digests, files, etc.

use std::fs;
use std::io;
use std::path::Path;

use ed25519_compact::{KeyPair, PublicKey, SecretKey};
use filebuffer::FileBuffer;
use sha2::Sha256;

use error::{Error, Result};
use format;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// String-format bytes as lowercase hexadecimal, append to the string.
pub fn append_hex(string: &mut String, bytes: &[u8]) {
    for &b in bytes {
        string.push(HEX_CHARS[(b >> 4) as usize]);
        string.push(HEX_CHARS[(b & 0xf) as usize]);
    }
}

/// Sha256 digest of some input.
///
/// Note, the `Eq` impl is not constant time. This is not an issue for Tako,
/// because verification of the digest happens client-side; there is no server
/// logic.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Digest([u8; 32]);

impl Digest {
    pub fn new(bytes: [u8; 32]) -> Digest {
        Digest(bytes)
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }

    #[cfg(test)]
    pub fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

/// Compute the SHA256 digest of a file. Mmaps the file.
pub fn sha256sum(path: &Path) -> Result<Digest> {
    use sha2::Digest;
    // Mmap the file when computing its digest. This way we can compute the
    // digest of files that don't fit in memory, without having to care about
    // streaming manually. Simple and fast.
    let fbuffer = FileBuffer::open(path)?;
    Ok(Digest(Sha256::digest(&fbuffer).into()))
}

/// Parse key pair as formatted by `format_key_pair()`.
pub fn parse_key_pair(pair_base64: &str) -> Result<KeyPair> {
    // To stress that the secret key is secret, we always prefix it with
    // "SECRET+", to hopefully make users think twice before pasting that key
    // into a terminal with Bash history enabled, or before saving it to a plain
    // text file. If the prefix is not there, reject the key pair.
    match &pair_base64[..7] {
        "SECRET+" => { /* Ok, as expected. */ }
        _ => return Err(Error::InvalidSecretKeyPrefix),
    }

    let err = Error::InvalidSecretKeyData;
    let pair_bytes = format::decode_base64(&pair_base64[7..]).ok_or(err)?;

    // The key pair printed to the user is the concatenation of the private key
    // (64 bytes) and public key (32 bytes).
    if pair_bytes.len() != 96 {
        return Err(Error::InvalidSecretKeyData);
    }

    let err = Error::InvalidSecretKeyData;
    let secret_key = SecretKey::from_slice(&pair_bytes[..64]).map_err(|_| err)?;

    let err = Error::InvalidSecretKeyData;
    let public_key = PublicKey::from_slice(&pair_bytes[64..]).map_err(|_| err)?;

    let keypair = KeyPair { pk: public_key, sk: secret_key };
    Ok(keypair)
}

/// Format key pair as base64 string with "SECRET+" prefix.
pub fn format_key_pair(key_pair: &KeyPair) -> String {
    // We prefix the secret key with "SECRET+" everywhere to stress its secrecy;
    // we expect that same prefix when reading it back. Use "+" rather than ":"
    // as separator, because Gnome Terminal selects the entire line on double
    // click with "+" but not with ":", and also because a user might think that
    // a "SECRET:" prefix is just a label and not part of the key, whereas with
    // a "+" as separator it looks more like one thing.
    let mut pair_bytes = Vec::with_capacity(96);
    pair_bytes.extend_from_slice(&key_pair.sk[..]);
    pair_bytes.extend_from_slice(&key_pair.pk[..]);

    let mut result = String::with_capacity(128 + 7);
    result.push_str("SECRET+");
    format::append_base64(&mut result, &pair_bytes);
    result
}

/// A file that is deleted on drop, unless explicitly renamed.
///
/// This is used to write to a temporary file, which is cleaned up automatically
/// on an error: construct a `FileGuard` with the file path. In case of an early
/// return due to an error, the guard goes out of scope and deletes the file. If
/// the full write was successful, call `move_readonly()` to mark the file
/// read-only and move it into its final destination.
pub struct FileGuard<'a> {
    path: &'a Path,
    delete: bool,
}

impl<'a> FileGuard<'a> {
    pub fn new(path: &'a Path) -> FileGuard<'a> {
        FileGuard {
            path: path,
            delete: true,
        }
    }

    pub fn move_readonly(mut self, dest: &Path) -> io::Result<()> {
        // Make the file readonly.
        let mut perms = fs::metadata(self.path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(self.path, perms)?;
        fs::rename(self.path, dest)?;
        self.delete = false;
        Ok(())
    }
}

impl<'a> Drop for FileGuard<'a> {
    fn drop(&mut self) {
        if self.delete {
            // Remove the temp file. The drop with `delete` set happens on an
            // error path, so the file is likely incomplete, or its signature or
            // digest might be invalid. Removing the file is an operation that
            // may fail, but we are already in a failure mode, and deleting the
            // temp file is part of error recovery. If recovery fails, the
            // original error is more informative than the secondary IO error.
            // Besides, we cannot return the error here anyway. So ignore the
            // secondary error.
            let _ = fs::remove_file(self.path);
        }
    }
}

#[cfg(test)]
mod test {
    use ed25519_compact::KeyPair;

    use error::Error;
    use super::{format_key_pair, parse_key_pair};

    #[test]
    fn format_key_pair_then_parse_key_pair_is_identity() {
        for _ in 0..1024 {
            let key_pair_in = KeyPair::generate();
            let formatted = format_key_pair(&key_pair_in);
            let key_pair_out = parse_key_pair(&formatted).unwrap();
            assert_eq!(key_pair_in, key_pair_out);
        }
    }

    #[test]
    fn parse_key_pair_requires_prefix() {
        match parse_key_pair("R1/aB01J60F3fPk7") {
            Err(Error::InvalidSecretKeyPrefix) => { /* Expected */ }
            _ => panic!("Should have returned invalid prefix error."),
        }
    }

    #[test]
    fn parse_key_pair_rejects_input_that_is_too_short() {
        match parse_key_pair("SECRET+R1/aB01J60F3fPk7") {
            Err(Error::InvalidSecretKeyData) => { /* Expected */ }
            _ => panic!("Should have returned invalid secret key data error."),
        }
    }
}
