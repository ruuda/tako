// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Errors that Tako can encounter.

use std::io;
use std::result;

use hyper::error::UriError;

pub enum Error {
    /// Error in config file on a given line.
    InvalidConfig(usize, &'static str),

    /// Invalid URI in config file on a given line.
    InvalidUri(usize, UriError),

    /// A key is missing in the config.
    IncompleteConfig(&'static str),

    /// IO error.
    IoError(io::Error),
}

pub type Result<T> = result::Result<T, Error>;

// TODO: Implement std::error::Error for Error.
