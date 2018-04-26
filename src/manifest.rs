// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Manifest file parser.

use std::cmp::Ordering;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str;

use base64;
use ring::signature;
use ring::signature::Ed25519KeyPair;
use untrusted::Input;

use config::PublicKey;
use error::{Error, Result};
use util;
use version::Version;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Sha256([u8; 32]);

impl Sha256 {
    pub fn copy_from_slice(bytes: &[u8]) -> Sha256 {
        let mut sha256 = [0_u8; 32];
        sha256.copy_from_slice(bytes);
        Sha256(sha256)
    }
}

impl AsRef<[u8]> for Sha256 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub version: Version,
    pub digest: Sha256,
}

// Implement Ord manually for Entry; the generated one would also compare
// digests, which is wasteful, because we should not have duplicate versions.

impl Ord for Entry {
    fn cmp(&self, other: &Entry) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Manifest {
    entries: Vec<Entry>,
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
        version: Version::new(version),
        digest: Sha256(sha256),
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
    pub fn new() -> Manifest {
        Manifest {
            entries: Vec::new(),
        }
    }

    pub fn parse(bytes: &[u8], public_key: &PublicKey) -> Result<Manifest> {
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
        let signature_bytes = parse_signature(signature_line)?;

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

        // The signature and newline are 89 bytes. Everything before that is
        // included in the signature.
        let message = Input::from(&bytes[..bytes.len() - 89]);
        let pub_key = public_key.as_input();
        let sig = Input::from(&signature_bytes);

        if signature::verify(&signature::ED25519, pub_key, message, sig).is_err() {
            return Err(Error::InvalidSignature)
        }

        let manifest = Manifest {
            entries: entries,
        };

        Ok(manifest)
    }

    /// Return whether all entries of self also occur in other.
    pub fn is_subset_of(&self, other: &Manifest) -> bool {
        let mut entries_other = other.entries.iter();

        // Because we assume that entries in the manifest are sorted, we can do
        // a mergesort-like check for subset: all the entries in self.entries
        // should eventually occur in other.entries. The other manifest can have
        // more, but then we just skip over them.
        for entry in &self.entries {
            loop {
                match entries_other.next() {
                    Some(ref e) if *e == entry => break,
                    Some(..) => continue,
                    None => return false,
                }
            }
        }

        true
    }

    /// Print the manifest as a string and sign it, the inverse of `parse`.
    pub fn serialize(&self, key_pair: &Ed25519KeyPair) -> String {
        // Premature optimization: estimate the output size, so we have to do
        // only a single allocation. 18 bytes for header (including newlines),
        // 64 bytes per entry for the hash, 15 for version, space, and newline.
        // And then 90 bytes for the signature including newlines.
        let n = 18 + self.entries.len() * (15 + 64) + 90;
        let mut out = String::with_capacity(n);

        out.push_str("Tako Manifest 1\n\n");
        for entry in &self.entries {
            out.push_str(entry.version.as_str());
            out.push(' ');
            util::append_hex(&mut out, &entry.digest.as_ref());
            out.push('\n');
        }

        out.push('\n');

        let signature = key_pair.sign(out.as_bytes());
        let signature_b64 = base64::encode(signature.as_ref());

        out.push_str(&signature_b64);
        out.push('\n');

        out
    }

    /// Load a locally stored manifest from a store directory.
    ///
    /// If the manifest exists, it is parsed and returned. If it does not exist,
    /// None is returned, rather than an Err.
    pub fn load_local(dir: &Path, public_key: &PublicKey) -> Result<Option<Manifest>> {
        // Open the current manifest. If it does not exist that is not an error.
        let mut path = PathBuf::from(dir);
        path.push("manifest");
        let mut f = match fs::File::open(path) {
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            other => other?,
        };

        let mut manifest_bytes = Vec::new();
        f.read_to_end(&mut manifest_bytes)?;

        Ok(Some(Manifest::parse(&manifest_bytes[..], public_key)?))
    }

    /// Insert a new entry, keeping the entries ordered.
    pub fn insert(&mut self, entry: Entry) -> Result<()> {
        match self.entries.binary_search(&entry) {
            Ok(i) => {
                if self.entries[i].digest != entry.digest {
                    return Err(Error::Duplicate(entry.version))
                }
                if self.entries[i].version.as_str() != entry.version.as_str() {
                    return Err(Error::Duplicate(entry.version))
                }
                // The version existed already, but it is identical to what we
                // are trying to insert, so that is fine.
            }
            Err(i) => self.entries.insert(i, entry),
        }
        Ok(())
    }

    /// Return the entry with the largest version number that is within bounds.
    ///
    /// The lower bound is inclusive and the upper bound is exclusive.
    pub fn latest_compatible_entry(&self, lower: &Version, upper: &Version) -> Option<&Entry> {
        // Entries are sorted by ascending version, so we iterate backwards to
        // find the latest applicable one.
        self.entries
            .iter()
            .rev()
            .filter(|e| *lower <= e.version && e.version < *upper)
            .next()
    }
}

/// Store a manifest locally. Writes first and then swaps the file.
///
/// Takes the target directory path and manifest bytes.
pub fn store_local(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut path_tmp = PathBuf::from(path);
    let mut path_final = PathBuf::from(path);
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

#[cfg(test)]
mod test {
    use ring::signature::Ed25519KeyPair;
    use ring::test::rand::FixedSliceRandom;
    use untrusted::Input;

    use config::PublicKey;
    use error::Error;
    use super::{Entry, Manifest, Sha256, parse_entry};
    use version::Version;

    fn get_test_key_pair() -> Ed25519KeyPair {
        // Produce the keypair from the same 32 bytes each time in the tests,
        // so they are deterministic. From this seed, the following key is
        // generated:
        // Secret key: MFMCAQEwBQYDK2VwBCIEIHRlc3Qta2V5LXZlcnktc2VjdXJpdHktc3Vja
        // C1zYWZloSMDIQCXQPbwnZ+Ihe9Y9t5k/vCRqr50HnkaXbKyKCX2ZAfb2Q==
        // Public key: l0D28J2fiIXvWPbeZP7wkaq+dB55Gl2ysigl9mQH29k=
        let seed = b"test-key-very-security-such-safe";
        let rng = FixedSliceRandom { bytes: &seed[..] };
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        Ed25519KeyPair::from_pkcs8(Input::from(&pkcs8_bytes)).unwrap()
    }

    fn get_test_public_key() -> PublicKey {
        PublicKey::from_pair(&get_test_key_pair())
    }

    /// A sequence of 32 bytes that I don't want to repeat everywhere.
    fn get_test_sha256() -> Sha256 {
        const TEST_SHA256: [u8; 32] = [
            0x96, 0x41, 0xa4, 0x9d, 0x02, 0xe9, 0x0c, 0xbb, 0x62, 0x13, 0xf2,
            0x02, 0xfb, 0x63, 0x2d, 0xa7, 0x0c, 0xdc, 0x59, 0x07, 0x3d, 0x42,
            0x28, 0x3c, 0xfc, 0xdc, 0x1d, 0x78, 0x64, 0x54, 0xf1, 0x7f
        ];
        Sha256(TEST_SHA256)
    }

    fn get_test_entry(version: &'static str) -> Entry {
        Entry {
            version: Version::from(version),
            digest: get_test_sha256(),
        }
    }

    #[test]
    fn parse_entry_parses_entry() {
        let raw = b"1.1.0 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f";
        let entry = parse_entry(&raw[..]).unwrap();
        assert_eq!(&entry.version.as_str(), &"1.1.0");
        assert_eq!(entry.digest, get_test_sha256());
    }

    #[test]
    fn parse_rejects_unknown_version() {
        let raw = b"Tako Manifest 1.1\n\nWrong!\n";
        match Manifest::parse(&raw[..], &get_test_public_key()) {
            Err(Error::InvalidManifest(..)) => { /* This is expected. */ },
            _ => panic!("Manifest should be rejected."),
        }
    }

    #[test]
    fn parse_parses_single_entry_manifest() {
        let raw = b"Tako Manifest 1\n\n\
            1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2\n\n\
            R9fjMZ9e2c5IrfByS53H6ur0VSWQfdTgAS2Y3t3lYcH9+ogDGtrbe65GhgEmDDD20Gfy8VyZQ82byF+NSANwDg==\n";
        let manifest = Manifest::parse(&raw[..], &get_test_public_key()).unwrap();
        assert_eq!(manifest.entries.len(), 1);
    }

    #[test]
    fn parse_rejects_manifest_on_signature_verification_failure() {
        // The raw data here is identical to that in the test above apart from
        // the signature. The data above has a correct signature, so the
        // signature here must be wrong.
        let raw = b"Tako Manifest 1\n\n\
            1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2\n\n\
            fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==\n";
        match Manifest::parse(&raw[..], &get_test_public_key()) {
            Err(Error::InvalidSignature) => { /* This is expected. */ },
            _ => panic!("Manifest should be rejected."),
        }
    }

    #[test]
    fn parse_parses_double_entry_manifest() {
        let raw = b"Tako Manifest 1\n\n\
            1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2\n\
            2.0.0 b7b01c6f6772529c66b945e559cb1f46546ef62063e44c1d1068725157ae1cda\n\n\
            LxHj9lwxekDPgmZmhutklX65IZNV8KAVDEncot9JEo0Spsr2FVlcWkId7IFHwvR+5lxcKVxIAcgz3pf0vC7ABQ==\n";
        let manifest = Manifest::parse(&raw[..], &get_test_public_key()).unwrap();
        assert_eq!(manifest.entries.len(), 2);
    }

    // TODO: Add fuzzer for manifest parser. It is quite straightforward to do
    // so with cargo-fuzz.

    #[test]
    fn serialize_outputs_manifest() {
        let entry = get_test_entry("1.0.0");
        let manifest = Manifest {
            entries: vec![entry],
        };
        let serialized = manifest.serialize(&get_test_key_pair());
        let expected = "Tako Manifest 1\n\n\
            1.0.0 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f\n\n\
            ttye/o4X1aOQQwk8Rf9OHLyqhfhi440qgH8cxw8ol/UgoSj7e1tQbhoA44Q+vEonigVwPMl82j6T0X7hTbziAQ==\n";
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_then_parse_is_identity() {
        let entry = get_test_entry("1.0.0");
        let manifest = Manifest {
            entries: vec![entry],
        };
        let serialized = manifest.serialize(&get_test_key_pair());
        let deserialized = Manifest::parse(
            serialized.as_bytes(),
            &get_test_public_key()
        ).unwrap();
        assert_eq!(deserialized, manifest);
    }

    #[test]
    fn entry_order_does_not_depend_on_insertion_order() {
        let entry0 = get_test_entry("0.0.0");
        let entry1 = get_test_entry("1.0.0");

        let mut m_0_1 = Manifest::new();
        m_0_1.insert(entry0.clone()).unwrap();
        m_0_1.insert(entry1.clone()).unwrap();

        let mut m_1_0 = Manifest::new();
        m_1_0.insert(entry1).unwrap();
        m_1_0.insert(entry0).unwrap();

        assert_eq!(m_0_1, m_1_0);
    }

    #[test]
    fn insert_allows_reinsert_if_identical() {
        let entry = get_test_entry("0.0.0");
        let mut manifest = Manifest::new();
        manifest.insert(entry.clone()).unwrap();
        manifest.insert(entry).unwrap();
        assert_eq!(manifest.entries.len(), 1);
    }

    #[test]
    fn insert_rejects_reinsert_if_digest_differs() {
        let entry = get_test_entry("0.0.0");
        let mut entry_alt = entry.clone();
        // Change the digest.
        entry_alt.digest.0[8] = 144;

        let mut manifest = Manifest::new();
        manifest.insert(entry).unwrap();
        match manifest.insert(entry_alt) {
            Err(Error::Duplicate(ref v)) if *v == Version::from("0.0.0") => {
                // This is expected.
            },
            _ => panic!("Insert should be rejected."),
        }
    }

    #[test]
    fn insert_rejects_reinsert_if_version_format_differs() {
        let entry = get_test_entry("1.0.0");
        let entry_alt = get_test_entry("1.0-0");

        let mut manifest = Manifest::new();
        manifest.insert(entry).unwrap();
        match manifest.insert(entry_alt) {
            Err(Error::Duplicate(ref v)) if *v == Version::from("1.0-0") => {
                // This is expected.
            },
            _ => panic!("Insert should be rejected."),
        }
    }

    #[test]
    fn latest_compatible_entry_finds_entry() {
        let manifest = Manifest {
            entries: vec![
                get_test_entry("0.0.0"),
                get_test_entry("0.1.0"),
                get_test_entry("0.2.0"),
                get_test_entry("1.0.0"),
                get_test_entry("1.0.0-beta.1"),
                get_test_entry("1.0.0-beta.2"),
                get_test_entry("1.1.0"),
                get_test_entry("1.2.0"),
                get_test_entry("1.2.1"),
                get_test_entry("2.0.0"),
            ],
        };

        let (u, w) = Version::from("*").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w).unwrap();
        assert_eq!(entry.version, Version::from("2.0.0"));

        let (u, w) = Version::from("0.*").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w).unwrap();
        assert_eq!(entry.version, Version::from("0.2.0"));

        let (u, w) = Version::from("1.*").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w).unwrap();
        assert_eq!(entry.version, Version::from("1.2.1"));

        // TODO: Perhaps the least number of parts should sort last, to avoid
        // this kind of issue? Although that would put 1.0 after 1.0.1 ... But
        // then again, don't do that. Name the first version 1.0.0 then.
        let (u, w) = Version::from("1.0.*").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w).unwrap();
        assert_eq!(entry.version, Version::from("1.0.0-beta.2"));

        let (u, w) = Version::from("1.2.0").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w).unwrap();
        assert_eq!(entry.version, Version::from("1.2.0"));

        let (u, w) = Version::from("3.*").pattern_to_bounds();
        let entry = manifest.latest_compatible_entry(&u, &w);
        assert!(entry.is_none());
    }
}
