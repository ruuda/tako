// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Command-line argument parser.
//!
//! There do exist Rust libraries for this, but they either bring along too many
//! dependencies, or they only support flags and not commands.

use std::env;
use std::env::Args;

const USAGE: &'static str = "
Tako -- Take container image.

Usage:
  tako fetch [--init] [--] <config>...
  tako store ---key=<keyfile> --output=<dir> <image> <version>
  tako -h | --help
  tako --version

Options:
  --init              Download image only if none exists already.
  -k --key=<keyfile>  Private key to sign the manifest with.
  -o --output=<dir>   Server target directory.
  -h --help           Show this screen.
  --version           Show version.
";

pub enum Cmd {
    Fetch(Vec<String>),
    Init(Vec<String>),
    Store(String, String, String, String),
    Help,
    Version,
}

pub fn print_usage() {
    print!("{}", USAGE);
}

pub fn parse() -> Result<Cmd, String> {
    let mut args = env::args();

    // Skip executable name.
    args.next();

    match args.next().as_ref().map(|s| &s[..]) {
        Some("fetch") => parse_fetch(args),
        Some("store") => parse_store(args),
        Some("-h") | Some("--help") => drain(args).and(Ok(Cmd::Help)),
        Some("--version") => drain(args).and(Ok(Cmd::Version)),
        Some(other) => unexpected(other),
        None => Err("No command provided. See --help.".to_string()),
    }
}

fn parse_fetch(mut args: Args) -> Result<Cmd, String> {
    let mut fnames = Vec::new();
    let mut is_init = false;
    let mut is_raw = false;
    for arg in args {
        if is_raw {
            fnames.push(arg);
            continue
        }
        match &arg[..] {
            "--init" => {
                is_init = true;
                continue
            }
            "--" => {
                is_raw = true;
                continue
            }
            other if other.starts_with("--") => return unexpected(other),
            _ => {}
        }
        fnames.push(arg);
    }

    if fnames.len() == 0 {
        return Err("Expected at least one filename of fetch config.".to_string())
    }

    if is_init {
        Ok(Cmd::Init(fnames))
    } else {
        Ok(Cmd::Fetch(fnames))
    }
}

fn parse_store(mut args: Args) -> Result<Cmd, String> {
    unimplemented!();
}

fn drain(mut args: Args) -> Result<(), String> {
    for arg in args {
        return unexpected::<()>(&arg);
    }

    Ok(())
}

fn unexpected<T>(arg: &str) -> Result<T, String> {
    Err(format!("Unexpected argument '{}'.", arg))
}
