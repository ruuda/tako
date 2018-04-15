// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Manifest file parser.

use base64;

use error::{Error, Result};

pub struct Entry {
    pub version: String,
    pub sha256: [u8; 32],
}

pub struct Manifest {
    pub entries: Vec<Entry>,
    pub signature: [u8; 64],
}

fn parse_signature(sig_base64: &[u8]) -> Result<[u8; 64]> {
    let bytes = match base64::decode(sig_base64) {
        Ok(bs) => bs,
        Err(err) => return Err(Error::InvalidSignatureData(err)),
    };

    if bytes.len() != 64 {
        let msg = "Ed25519 signature is not 64 bytes (88 characters base64).";
        return Err(Error::InvalidManifest(msg))
    }

    let mut result = [0_u8; 64];
    result.copy_from_slice(&bytes[..]);

    Ok(result)
}

impl Manifest {
    pub fn parse(bytes: &[u8]) -> Result<Manifest> {
        // 88 bytes of base64 signature, then a newline.
        if bytes.len() < 89 {
            let msg = "Manifest is too short to contain even the signature.";
            return Err(Error::InvalidManifest(msg))
        }

        let signature = parse_signature(&bytes[..88])?;
        let _data = &bytes[89..];

        // TODO: Parse entries.

        let manifest = Manifest {
            entries: Vec::new(),
            signature: signature,
        };

        Ok(manifest)
    }
}
