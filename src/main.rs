// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// TODO: Disallow when the pieces come together.
#![allow(dead_code)]

// TODO: Use the system allocator (not jemalloc), when that makes it into Rust
// stable. See also this excellent binary size guide:
// https://jamesmunns.com/blog/tinyrocket/
//
// #![feature(alloc_system, global_allocator, allocator_api)]
// extern crate alloc_system;
//
// use alloc_system::System;
//
// #[global_allocator]
// static A: System = System;

extern crate base64;
extern crate filebuffer;
extern crate ring;
extern crate untrusted;

use std::io::Write;
use std::io;
use std::process;
use std::env;

use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use untrusted::Input;

mod cli;
mod config;
mod curl;
mod error;
mod fetch;
mod manifest;
mod store;
mod util;
mod version;

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

fn run_store(store: cli::Store) {
    store::store(store).unwrap();
}

fn run_gen_key() -> Result<(), ring::error::Unspecified> {
    // Generate a key pair in PKCS#8 (v2) format.
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;

    let key_pair = Ed25519KeyPair::from_pkcs8(Input::from(&pkcs8_bytes))?;

    // There is no particular reason to encode these as base64, apart from that
    // it is easy to deal with in config files (for the public key), and it can
    // be safely printed to stdout and copied from there.
    let secret_key_b64 = base64::encode(&pkcs8_bytes[..]);
    let public_key_b64 = base64::encode(key_pair.public_key_bytes());

    // Print the private key to stdout, rather than writing it to a file. This
    // means that at least the sensitive data is not written to disk. (It is
    // visible to spies looking over your shoulder, but I think that is less
    // likely than a malicious entity having filesystem access.) The user can
    // still decide to write the key to a file, or to put it in a secret store
    // like Vault. To sign the manifest, the secret can be pulled from Vault and
    // brought into the environment; it never needs to be written to disk except
    // encrypted.
    println!("Secret key (save to an encrypted secret store):\n{}", secret_key_b64);
    println!("\nPublic key:\n{}", public_key_b64);

    Ok(())
}

fn main() {
    use cli::Cmd;
    let args = env::args().collect();
    match cli::parse(args) {
        Ok(Cmd::Fetch(fnames)) => fnames.iter().for_each(run_fetch),
        Ok(Cmd::Init(fnames)) => fnames.iter().for_each(run_init),
        Ok(Cmd::Store(store)) => run_store(store),
        // TODO: Implement a better error handler.
        Ok(Cmd::GenKey) => run_gen_key().unwrap(),
        Ok(Cmd::Help(cmd)) => cli::print_usage(cmd),
        Ok(Cmd::Version) => cli::print_version(),
        Err(msg) => {
            println!("{}", msg); // TODO: stderr.
            process::exit(1);
        }
    }
}
