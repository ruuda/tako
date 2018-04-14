// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Configuration file parser.

use std::str::FromStr;
use std::path::PathBuf;

use hyper::Uri;

use error::{Error, Result};

struct Config {
    origin: Uri,
    public_key: [u8; 32],
    destination: PathBuf,
    restart_units: Vec<String>,
}

impl Config {
    pub fn parse<'a, I>(mut lines: I) -> Result<Config>
    where I: Iterator<Item = &'a str> {
        let mut origin = None;
        let mut public_key = None;
        let mut destination = None;
        let mut restart_units = Vec::new();

        for (lineno, line) in lines.enumerate() {
            // Allow empty lines in the config file.
            if line.len() == 0 {
                continue
            }

            if let Some(n) = line.find('=') {
                let key = &line[..n];
                let value = &line[n + 1..];
                match key {
                    "Origin" => {
                        match Uri::from_str(value) {
                            Ok(uri) => origin = Some(uri),
                            Err(err) => return Err(Error::InvalidUri(lineno, err)),
                        }
                    }
                    "PublicKey" => {
                        unimplemented!("TODO: Parse base64 key.");
                    }
                    "Destination" => {
                        destination = Some(PathBuf::from(value));
                    }
                    "RestartUnit" => {
                        restart_units.push(String::from(value));
                    }
                    _ => {
                        let msg = "Unknown key. Expected 'Origin', 'PublicKey', 'Destination', or 'RestartUnit'.";
                        return Err(Error::InvalidConfig(lineno, msg))
                    }
                }
            } else {
                let msg = "Line contains no '='. Expected 'Origin=https://example.com'-like key-value pair.";
                return Err(Error::InvalidConfig(lineno, msg))
            }
        }

        let config = Config {
            origin: match origin {
                Some(o) => o,
                None => return Err(Error::IncompleteConfig("Origin not set. Expected 'Origin='-line.")),
            },
            public_key: match public_key {
                Some(k) => k,
                None => return Err(Error::IncompleteConfig("Public key not set. Expected 'PublicKey='-line.")),
            },
            destination: match destination {
                Some(d) => d,
                None => return Err(Error::IncompleteConfig("Destination not set. Expected 'Destination=/path'-line.")),
            },
            restart_units: restart_units,
        };

        Ok(config)
    }
}
