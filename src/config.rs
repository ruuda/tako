// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Configuration file parser.

use std::path::PathBuf;

use base64;
use sodiumoxide::crypto::sign::ed25519;

use error::{Error, Result};
use version::Version;

#[derive(Debug)]
pub struct Config {
    pub origin: String,
    pub public_key: ed25519::PublicKey,
    pub version: Version,
    pub destination: PathBuf,
    pub restart_units: Vec<String>,
}

fn parse_public_key(lineno: usize, key_base64: &str) -> Result<[u8; 32]> {
    let bytes = match base64::decode(key_base64) {
        Ok(bs) => bs,
        Err(err) => return Err(Error::InvalidPublicKeyData(lineno, err)),
    };

    if bytes.len() != 32 {
        let msg = "Ed25519 public key is not 32 bytes (44 characters base64).";
        return Err(Error::InvalidConfig(lineno, msg))
    }

    let mut result = [0_u8; 32];
    result.copy_from_slice(&bytes[..]);

    Ok(result)
}

impl Config {
    pub fn parse<'a, I, S>(lines: I) -> Result<Config>
    where I: IntoIterator<Item = S>,
          S: AsRef<str> {
        let mut origin = None;
        let mut public_key = None;
        let mut version = None;
        let mut destination = None;
        let mut restart_units = Vec::new();

        for (lineno, line_raw) in lines.into_iter().enumerate() {
            let line = line_raw.as_ref();

            // Allow empty lines in the config file.
            if line.len() == 0 {
                continue
            }

            // Skip lines starting with '#' or ';' to allow comments. This is
            // consistent with systemd's comment syntax.
            if line.starts_with("#") || line.starts_with(";") {
                continue
            }

            if let Some(n) = line.find('=') {
                let key = &line[..n];
                let value = &line[n + 1..];
                match key {
                    "Origin" => {
                        origin = Some(String::from(value));
                    }
                    "PublicKey" => {
                        public_key = Some(parse_public_key(lineno, value)?);
                    }
                    "Version" => {
                        version = Some(Version::from(value));
                    }
                    "Destination" => {
                        destination = Some(PathBuf::from(value));
                    }
                    "Restart" => {
                        for unit in value.split(|ch| ch == ' ') {
                            restart_units.push(String::from(unit));
                        }
                    }
                    _ => {
                        let msg = "Unknown key. Expected one of \
                            'Origin', 'PublicKey', 'Version', 'Destination', \
                            or 'Restart'.";
                        return Err(Error::InvalidConfig(lineno, msg))
                    }
                }
            } else {
                let msg = "Line contains no '='. \
                    Expected 'Origin=https://example.com'-like key-value pair.";
                return Err(Error::InvalidConfig(lineno, msg))
            }
        }

        let config = Config {
            origin: match origin {
                Some(o) => o,
                None => return Err(Error::IncompleteConfig(
                    "Origin not set. Expected 'Origin='-line."
                )),
            },
            public_key: match public_key {
                Some(k) => ed25519::PublicKey(k),
                None => return Err(Error::IncompleteConfig(
                    "Public key not set. Expected 'PublicKey='-line."
                )),
            },
            version: match version {
                Some(v) => v,
                None => return Err(Error::IncompleteConfig(
                    "Version not set. Expected 'Version='-line. \
                    Use 'Version=*' to accept any version."
                )),
            },
            destination: match destination {
                Some(d) => d,
                None => return Err(Error::IncompleteConfig(
                    "Destination not set. Expected 'Destination=/path'-line."
                )),
            },
            restart_units: restart_units,
        };

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::Config;
    use version::Version;

    #[test]
    pub fn config_with_0_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Destination=/var/lib/images/app-foo",
            "Version=*",
        ];
        let config = Config::parse(&config_lines).unwrap();
        assert_eq!(&config.origin[..], "https://images.example.com/app-foo");
        assert_eq!(config.public_key.0[..4], [0xf3, 0xea, 0xf9, 0x0c]);
        assert_eq!(config.destination.as_path(), Path::new("/var/lib/images/app-foo"));
        assert_eq!(config.version, Version::from("*"));
    }

    #[test]
    pub fn config_with_1_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Version=*",
            "Destination=/var/lib/images/app-foo",
            "Restart=foo",
        ];
        let config = Config::parse(&config_lines).unwrap();
        assert_eq!(&config.restart_units[..], &["foo"]);
    }

    #[test]
    pub fn config_with_2_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Version=1.*",
            "Destination=/var/lib/images/app-foo",
            "Restart=foo",
            "Restart=bar",
        ];
        let config = Config::parse(&config_lines).unwrap();
        assert_eq!(&config.restart_units[..], &["foo", "bar"]);
    }

    #[test]
    pub fn config_with_2_space_separated_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Version=1.*",
            "Destination=/var/lib/images/app-foo",
            "Restart=foo bar",
        ];
        let config = Config::parse(&config_lines).unwrap();
        assert_eq!(&config.restart_units[..], &["foo", "bar"]);
    }

    #[test]
    pub fn config_with_3_mixed_separated_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Version=1.*",
            "Destination=/var/lib/images/app-foo",
            "Restart=foo bar",
            "Restart=baz",
        ];
        let config = Config::parse(&config_lines).unwrap();
        assert_eq!(&config.restart_units[..], &["foo", "bar", "baz"]);
    }

    #[test]
    pub fn parse_skips_comments() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "# This is a comment.",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "; This is also a comment.",
            "Destination=/var/lib/images/app-foo",
            "Version=1",
        ];
        assert!(Config::parse(&config_lines).is_ok());
    }

    // TODO: Test error cases.
}
