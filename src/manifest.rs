// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Manifest file parser.

use base64;

use error::{Error, Result};
use std::str;

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub version: String,
    pub sha256: [u8; 32],
}

pub struct Manifest {
    pub entries: Vec<Entry>,
    pub signature: [u8; 64],
}

/// Parse header and return the version number.
fn parse_header(header: &[u8]) -> Result<u32> {
    if header == b"Tako Manifest 1" {
        Ok(1)
    } else if header.starts_with(b"Tako Manifest") {
        let msg = "Manifest version is not supported.";
        Err(Error::InvalidManifest(msg))
    } else {
        let msg = "Manifest does not contain expected 'Tako Manifest 1' header.";
        Err(Error::InvalidManifest(msg))
    }
}

/// Parse a lowercase ascii byte in [0, ..., f], return value as int.
fn parse_hex(ch: u8) -> Option<u8> {
    if ch < b'0' { return None }
    if ch > b'f' { return None }
    if ch <= b'9' {
        Some(ch - b'0')
    } else if ch >= b'a' {
        Some(ch - b'a' + 10)
    } else {
        None
    }
}

/// Parse a single entry line.
fn parse_entry(line: &[u8]) -> Result<Entry> {
    let mid_opt = line.iter().cloned().enumerate().filter(|&(_, ch)| ch == b' ').next();
    let msg = "Invalid manifest entry, expected a space after version number.";
    let mid = mid_opt.map(|i_ch| i_ch.0).ok_or(Error::InvalidManifest(msg))?;
    let version_bytes = &line[..mid];
    let sha256_hex = &line[mid + 1..];

    let version = match str::from_utf8(version_bytes) {
        Ok(s) => s.to_string(),
        Err(..) => {
            let msg = "Entry version number is not valid UTF-8.";
            return Err(Error::InvalidManifest(msg))
        }
    };

    if sha256_hex.len() != 64 {
        let msg = "Entry hash is not 32 bytes (64 hexadecimal characters).";
        return Err(Error::InvalidManifest(msg))
    }

    let mut sha256 = [0_u8; 32];
    for (dst, hex) in sha256.iter_mut().zip(sha256_hex.chunks(2)) {
        // There is also u8::form_str_radix, but then we would need to do UTF-8
        // validation first, and all the error handling is just as messy as just
        // doing it manually. As an additional benefit, we are stricter to only
        // allow lowercase hexadecimal.

        // Indexing does not go out of bounds here because we verified the
        // length above.
        let msg = "Invalid entry hash. Must be lowercase hexadecimal.";
        let high = parse_hex(hex[0]).ok_or(Error::InvalidManifest(msg))?;
        let low = parse_hex(hex[1]).ok_or(Error::InvalidManifest(msg))?;
        *dst = (high << 4) + low;
    }

    let entry = Entry {
        version: version,
        sha256: sha256,
    };

    Ok(entry)
}

/// Parse the base64-encoded signature line.
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
        let mut lines = bytes.split(|b| *b == b'\n');
        let mut entries = Vec::new();


        // First up, a line with the header.
        let err_trunc = Error::InvalidManifest("Unexpected end of manifest.");
        let header = lines.next().ok_or(err_trunc)?;
        let _version = parse_header(header)?;

        // Then a blank line.
        let err_trunc = Error::InvalidManifest("Unexpected end of manifest.");
        if lines.next().ok_or(err_trunc)? != b"" {
            let msg = "Expected blank line after header line.";
            return Err(Error::InvalidManifest(msg))
        }

        // Then one version per line.
        for line in &mut lines {
            if line == b"" {
                // A blank line indicates the end of the manifest, only the
                // signature follows after that.
                break
            }

            entries.push(parse_entry(line)?);
        }

        let err_trunc = Error::InvalidManifest("Unexpected end of manifest.");
        let signature_line = lines.next().ok_or(err_trunc)?;
        let signature = parse_signature(signature_line)?;

        // We expect the file to end with a trailing newline, and nothing after
        // that.
        if lines.next() != Some(b"") {
            let msg = "Expected newline at end of manifest.";
            return Err(Error::InvalidManifest(msg))
        }
        if lines.next() != None {
            let msg = "Unexpected trailing data after manifest.";
            return Err(Error::InvalidManifest(msg))
        }

        let manifest = Manifest {
            entries: entries,
            signature: signature,
        };

        Ok(manifest)
    }
}

#[cfg(test)]
mod test {
    use error::Error;
    use super::{Manifest, parse_entry};

    #[test]
    fn parse_entry_parses_entry() {
        let raw = b"1.1.0 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f";
        let entry = parse_entry(&raw[..]).unwrap();
        assert_eq!(&entry.version[..], "1.1.0");
        assert_eq!(&entry.sha256[..], &[
            0x96, 0x41, 0xa4, 0x9d, 0x02, 0xe9, 0x0c, 0xbb, 0x62, 0x13, 0xf2,
            0x02, 0xfb, 0x63, 0x2d, 0xa7, 0x0c, 0xdc, 0x59, 0x07, 0x3d, 0x42,
            0x28, 0x3c, 0xfc, 0xdc, 0x1d, 0x78, 0x64, 0x54, 0xf1, 0x7f
        ]);
    }

    #[test]
    fn parse_rejects_unknown_version() {
        let raw = b"Tako Manifest 1.1\n\nWrong!\n";
        match Manifest::parse(&raw[..]) {
            Err(Error::InvalidManifest(..)) => { /* This is expected. */ },
            _ => panic!("Manifest should be rejected."),
        }
    }

    #[test]
    fn parse_parses_single_entry_manifest() {
        let raw = b"Tako Manifest 1\n\n\
            1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2\n\n\
            fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==\n";
        let manifest = Manifest::parse(&raw[..]).unwrap();
        assert_eq!(manifest.entries.len(), 1);
    }

    #[test]
    fn parse_parses_double_entry_manifest() {
        let raw = b"Tako Manifest 1\n\n\
            1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2\n\
            2.0.0 b7b01c6f6772529c66b945e559cb1f46546ef62063e44c1d1068725157ae1cda\n\n\
            fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==\n";
        let manifest = Manifest::parse(&raw[..]).unwrap();
        assert_eq!(manifest.entries.len(), 2);
    }

    // TODO: Add fuzzer for manifest parser. It is quite straightforward to do
    // so with cargo-fuzz.
}
