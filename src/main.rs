extern crate base64;
extern crate hyper;

use std::io::Write;
use std::io;

mod config;
mod error;
mod curl;

fn main() {
    let mut curl_handle = curl::Handle::new();
    curl_handle.download("https://hyper.rs", |chunk| {
        io::stdout().write_all(chunk).unwrap();
    }).unwrap();
    println!("Done.");
}
