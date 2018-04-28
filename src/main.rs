// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// TODO: Disallow when the pieces come together.
#![allow(dead_code)]

extern crate base64;

use std::io::Write;
use std::io;
use std::process;

mod cli;
mod config;
mod curl;
mod error;
mod fetch;
mod manifest;

fn run_init(config_fname: &String) {
    println!("Run for {}.", config_fname);

    let mut curl_handle = curl::Handle::new();
    curl_handle.download("https://hyper.rs", |chunk| {
        io::stdout().write_all(chunk).unwrap();
    }).unwrap();
    println!("Done.");
}

fn run_fetch(config_fname: &String) {
    println!("Run for {}.", config_fname);
    fetch::fetch(config_fname).unwrap();
}

fn main() {
    use cli::Cmd;
    match cli::parse() {
        Ok(Cmd::Fetch(fnames)) => fnames.iter().for_each(run_fetch),
        Ok(Cmd::Init(fnames)) => fnames.iter().for_each(run_init),
        Ok(Cmd::Store(..)) => unimplemented!(),
        Ok(Cmd::Help) => cli::print_usage(),
        Ok(Cmd::Version) => cli::print_version(),
        Err(msg) => {
            println!("{}", msg); // TODO: stderr.
            process::exit(1);
        }
    }
}
