// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Command-line argument parser.
//!
//! There do exist Rust libraries for this, but they either bring along too many
//! dependencies, or they only support flags and not commands. And even then, a
//! command line parser is of limited help: validation and interaction between
//! flags still involves a lot of probing the "parsed" flags. So rather than
//! using an external parser, this module provides a light abstraction `ArgIter`
//! to deal with the distinction between long and short flags, and arguments,
//! and a handwritten parser/validator on top based mostly on pattern matching.

use std::env;
use std::fmt;
use std::path::PathBuf;
use std::vec;

use version::Version;

const USAGE: &'static str = "
Tako -- Take container image.

Usage:
  tako <command> [<args>...]
  tako -h | --help
  tako --version

Commands:
  fetch      Download or update an image.
  store      Add a new image version to a server directory.
  gen-key    Generate a key pair for signing manifests.

Options:
  -h --help  Show this screen, or help about a command.
  --version  Show version.

See 'tako <command> --help' for information on a specific command.
";

const USAGE_FETCH: &'static str = "
tako fetch -- Download or update an image.

Usage:
  tako fetch [--init] [--] <config>...

Options:
  --init    Download images only if none exists already.

Arguments:
  <config>  Path to a config file that determines what to fetch.
";

const USAGE_STORE: &'static str = "
tako store -- Add a new image version to a server directory.

Usage:
  tako store [-k <key> | -f <file>] --output <dir> [--] <image> <version>

Options:
  -k --key <key>        Secret key to sign the manifest with. Can alternatively
                        be read from the TAKO_SECRET_KEY environment variable.
  -f --key-file <file>  File to read the secret key from.
  -o --output <dir>     Server directory.

Arguments:
  <image>               Path to image file to be stored.
  <version>             Version to store the image under.
";

const USAGE_GEN_KEY: &'static str = "
tako gen-key -- Generate a key pair for signing manifests.

Usage:
  tako gen-key
";

#[derive(Debug, Eq, PartialEq)]
pub struct Store {
    pub secret_key: Option<String>,
    pub secret_key_path: Option<PathBuf>,
    pub output_path: PathBuf,
    pub version: Version,
    pub image_path: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Cmd {
    Fetch(Vec<String>),
    Init(Vec<String>),
    Store(Store),
    GenKey,
    Help(String),
    Version,
}

pub fn print_usage(cmd: String) {
    // Slice usage strings from 1, to cut off the initial newline.
    match &cmd[..] {
        "tako" => print!("{}", &USAGE[1..]),
        "fetch" => print!("{}", &USAGE_FETCH[1..]),
        "store" => print!("{}", &USAGE_STORE[1..]),
        "gen-key" => print!("{}", &USAGE_GEN_KEY[1..]),
        _ => println!("'{}' is not a Tako command. See 'tako --help'.", cmd),
    }
}

pub fn print_version() {
    println!("0.0.0");
    // TODO: Licenses and stuff.
}

enum Arg<T> {
    Plain(T),
    Short(T),
    Long(T),
}

impl Arg<String> {
    fn as_ref(&self) -> Arg<&str> {
        match *self {
            Arg::Plain(ref x) => Arg::Plain(&x[..]),
            Arg::Short(ref x) => Arg::Short(&x[..]),
            Arg::Long(ref x) => Arg::Long(&x[..]),
        }
    }

    fn into_string(self) -> String {
        match self {
            Arg::Plain(x) => x,
            Arg::Short(x) => x,
            Arg::Long(x) => x,
        }
    }
}

impl fmt::Display for Arg<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arg::Plain(ref x) => write!(f, "{}", x),
            Arg::Short(ref x) => write!(f, "-{}", x),
            Arg::Long(ref x) => write!(f, "--{}", x),
        }
    }
}

struct ArgIter {
    /// Underlying args iterator.
    args: vec::IntoIter<String>,

    /// Whether we have observed a `--` argument.
    is_raw: bool,

    /// Leftover to return after an `--foo=bar` or `-fbar`-style argument.
    ///
    /// `--foo=bar` is returned as `Long(foo)` followed by `Plain(bar)`.
    /// `-fbar` is returned as `Short(f)` followed by `Plain(bar)`.
    leftover: Option<String>,
}

impl ArgIter {
    pub fn new(args: Vec<String>) -> ArgIter {
        ArgIter {
            args: args.into_iter(),
            is_raw: false,
            leftover: None,
        }
    }
}

impl Iterator for ArgIter {
    type Item = Arg<String>;

    fn next(&mut self) -> Option<Arg<String>> {
        if self.leftover.is_some() {
            return self.leftover.take().map(Arg::Plain)
        }

        let arg = self.args.next()?;

        if self.is_raw {
            return Some(Arg::Plain(arg))
        }

        if &arg == "--" {
            self.is_raw = true;
            return self.next()
        }

        if arg.starts_with("--") {
            let mut flag = String::from(&arg[2..]);
            if let Some(i) = flag.find('=') {
                self.leftover = Some(flag.split_off(i + 1));
                flag.truncate(i);
            }
            return Some(Arg::Long(flag))
        }

        if arg.starts_with("-") {
            let mut flag = String::from(&arg[1..]);
            if flag.len() > 1 {
                self.leftover = Some(flag.split_off(1));
                flag.truncate(1);
            }
            return Some(Arg::Short(flag))
        }

        Some(Arg::Plain(arg))
    }
}

pub fn parse(argv: Vec<String>) -> Result<Cmd, String> {
    let mut args = ArgIter::new(argv);

    // Skip executable name.
    args.next();

    let arg = match args.next() {
        Some(a) => a,
        None => return Err("No command provided. See --help.".to_string()),
    };

    match arg.as_ref() {
        Arg::Plain("fetch") => parse_fetch(args),
        Arg::Plain("store") => parse_store(args),
        Arg::Plain("gen-key") => parse_gen_key(args),
        Arg::Long("version") => drain(args).and(Ok(Cmd::Version)),
        Arg::Short("h") | Arg::Long("help") => parse_help(args),
        _ => return unexpected(arg),
    }
}

fn parse_fetch(mut args: ArgIter) -> Result<Cmd, String> {
    let mut fnames = Vec::new();
    let mut is_init = false;
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            Arg::Plain(..) => fnames.push(arg.into_string()),
            Arg::Long("init") => is_init = true,
            Arg::Short("h") | Arg::Long("help") => return drain_help(args, "fetch"),
            _ => return unexpected(arg),
        }
    }

    if fnames.len() == 0 {
        return Err("Expected at least one fetch config filename.".to_string())
    }

    if is_init {
        Ok(Cmd::Init(fnames))
    } else {
        Ok(Cmd::Fetch(fnames))
    }
}

fn parse_store(mut args: ArgIter) -> Result<Cmd, String> {
    let mut output_path = None;
    let mut secret_key = None;
    let mut secret_key_path = None;
    let mut image_path = None;
    let mut version = None;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            Arg::Short("k") | Arg::Long("key") => {
                let msg = "Expected secret key after --key.";
                secret_key = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("f") | Arg::Long("key-file") => {
                let msg = "Expected key path after --key-file.";
                secret_key_path = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("o") | Arg::Long("output") => {
                let msg = "Expected server directory after --output.";
                output_path = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("h") | Arg::Long("help") => {
                return drain_help(args, "store")
            }
            Arg::Plain(..) if image_path.is_none() => {
                image_path = Some(arg.into_string());
            }
            Arg::Plain(..) if version.is_none() => {
                version = Some(arg.into_string());
            }
            _ => return unexpected(arg)
        }
    }

    // If --key nor --key-file are provided, check the TAKO_SECRET_KEY
    // environment variable.
    if secret_key.is_none() && secret_key_path.is_none() {
        match env::var("TAKO_SECRET_KEY") {
            Ok(v) => secret_key = Some(v),
            Err(..) => {
                let msg = "Secret key not provided. Pass it via --key, \
                           read if from a key file with --key-file, \
                           or set the TAKO_SECRET_KEY environment variable.";
                return Err(msg.to_string())
            }
        }
    }

    let msg = "Server directory not provided. Pass it via --output.";
    let output_path = output_path.ok_or(msg.to_string())?;

    let msg = "Image path not provided. See 'tako store --help' for usage.";
    let image_path = image_path.ok_or(msg.to_string())?;

    let msg = "Version not provided. See 'tako store --help' for usage.";
    let version = version.ok_or(msg.to_string())?;

    let store = Store {
        secret_key: secret_key,
        secret_key_path: secret_key_path.map(PathBuf::from),
        output_path: PathBuf::from(output_path),
        version: Version::new(version),
        image_path: PathBuf::from(image_path),
    };

    Ok(Cmd::Store(store))
}

fn parse_gen_key(mut args: ArgIter) -> Result<Cmd, String> {
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            Arg::Short("h") | Arg::Long("help") => return drain_help(args, "gen-key"),
            _ => return unexpected(arg),
        }
    }
    Ok(Cmd::GenKey)
}

fn parse_help(mut args: ArgIter) -> Result<Cmd, String> {
    match args.next() {
        Some(Arg::Plain(cmd)) => drain(args).and(Ok(Cmd::Help(cmd))),
        Some(arg) => unexpected(arg),
        None => Ok(Cmd::Help("tako".to_string())),
    }
}

fn drain_help(args: ArgIter, cmd: &'static str) -> Result<Cmd, String> {
    drain(args).and(Ok(Cmd::Help(cmd.to_string())))
}

fn expect_plain(args: &mut ArgIter, msg: &'static str) -> Result<String, String> {
    match args.next() {
        Some(Arg::Plain(a)) => Ok(a),
        Some(arg) => Err(format!("Unexpected argument '{}'. {}", arg, msg)),
        None => Err(msg.to_string()),
    }
}

fn drain(args: ArgIter) -> Result<(), String> {
    for arg in args {
        return unexpected::<()>(arg);
    }

    Ok(())
}

fn unexpected<T>(arg: Arg<String>) -> Result<T, String> {
    Err(format!("Unexpected argument '{}'. See 'tako --help'.", arg))
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use super::{Cmd, Store, parse};
    use version::Version;

    fn parse_slice(args: &[&'static str]) -> Result<Cmd, String> {
        let argv = args.iter().map(|s| String::from(*s)).collect();
        parse(argv)
    }

    #[test]
    fn parse_parses_help() {
        let expected = Ok(Cmd::Help("tako".to_string()));
        assert_eq!(parse_slice(&["tako", "-h"]), expected);
        assert_eq!(parse_slice(&["tako", "--help"]), expected);
    }

    #[test]
    fn parse_parses_cmd_help() {
        let fetch = Ok(Cmd::Help("fetch".to_string()));
        assert_eq!(parse_slice(&["tako", "-h", "fetch"]), fetch);
        assert_eq!(parse_slice(&["tako", "--help", "fetch"]), fetch);
        assert_eq!(parse_slice(&["tako", "fetch", "-h"]), fetch);
        assert_eq!(parse_slice(&["tako", "fetch", "--help"]), fetch);

        let store = Ok(Cmd::Help("store".to_string()));
        assert_eq!(parse_slice(&["tako", "-h", "store"]), store);
        assert_eq!(parse_slice(&["tako", "--help", "store"]), store);
        assert_eq!(parse_slice(&["tako", "store", "-h"]), store);
        assert_eq!(parse_slice(&["tako", "store", "--help"]), store);

        let gen_key = Ok(Cmd::Help("gen-key".to_string()));
        assert_eq!(parse_slice(&["tako", "-h", "gen-key"]), gen_key);
        assert_eq!(parse_slice(&["tako", "--help", "gen-key"]), gen_key);
        assert_eq!(parse_slice(&["tako", "gen-key", "-h"]), gen_key);
        assert_eq!(parse_slice(&["tako", "gen-key", "--help"]), gen_key);
    }

    #[test]
    fn parse_parses_fetch() {
        let fetch = Ok(Cmd::Fetch(vec!["foo".to_string(), "bar".to_string()]));
        assert_eq!(parse_slice(&["tako", "fetch", "foo", "bar"]), fetch);
        assert_eq!(parse_slice(&["tako", "fetch", "--", "foo", "bar"]), fetch);
        assert_eq!(parse_slice(&["tako", "fetch", "foo", "--", "bar"]), fetch);

        let fetch = Ok(Cmd::Fetch(vec!["foo".to_string(), "--bar".to_string()]));
        assert_eq!(parse_slice(&["tako", "fetch", "foo", "--", "--bar"]), fetch);
        assert_eq!(parse_slice(&["tako", "fetch", "--", "foo", "--bar"]), fetch);

        // Unexpected argument --bar or -D.
        assert!(parse_slice(&["tako", "fetch", "foo", "--bar"]).is_err());
        assert!(parse_slice(&["tako", "fetch", "-DFIRE_MISSILE", "foo"]).is_err());

        // No configs provided.
        assert!(parse_slice(&["tako", "fetch"]).is_err());
    }

    #[test]
    fn parse_parses_fetch_init() {
        let init = Ok(Cmd::Init(vec!["foo".to_string(), "bar".to_string()]));
        assert_eq!(parse_slice(&["tako", "fetch", "--init", "foo", "bar"]), init);
        assert_eq!(parse_slice(&["tako", "fetch", "foo", "--init", "bar"]), init);
        assert_eq!(parse_slice(&["tako", "fetch", "foo", "bar", "--init"]), init);
    }

    #[test]
    fn parse_parses_store() {
        let store = Store {
            secret_key: Some("secret".to_string()),
            secret_key_path: None,
            output_path: PathBuf::from("/tmp"),
            version: Version::from("3.7.5"),
            image_path: PathBuf::from("out.img"),
        };
        let expected = Ok(Cmd::Store(store));

        assert_eq!(parse_slice(
            &["tako", "store", "--output", "/tmp", "--key", "secret", "out.img", "3.7.5"]
        ), expected);
        assert_eq!(parse_slice(
            &["tako", "store", "--key", "secret", "--output", "/tmp", "out.img", "3.7.5"]
        ), expected);
        assert_eq!(parse_slice(
            &["tako", "store", "out.img", "3.7.5", "--key=secret", "-o", "/tmp"]
        ), expected);
        assert_eq!(parse_slice(
            &["tako", "store", "-ksecret", "out.img", "--output", "/tmp", "3.7.5"]
        ), expected);

        // Path and version not provided.
        assert!(parse_slice(
            &["tako", "store", "--output", "/tmp", "-ksecret", "out.img"]
        ).is_err());
        assert!(parse_slice(
            &["tako", "store", "--output", "/tmp", "-ksecret"]
        ).is_err());

        // Server directory not provided.
        assert!(parse_slice(
            &["tako", "store", "-ksecret", "out.img", "3.7.5"]
        ).is_err());

        // TODO: Verify --key-file/-f and environment variable getter.
    }
}
