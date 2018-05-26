// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Errors that Tako can encounter.

use std::io;
use std::result;

use base64;

use version::Version;

#[derive(Debug)]
pub enum Error {
    /// Error in config file on a given line.
    InvalidConfig(usize, &'static str),

    /// A key is missing in the config.
    IncompleteConfig(&'static str),

    /// Public key in config on a given line could not be parsed as base64.
    InvalidPublicKeyData(usize, base64::DecodeError),

    /// Secret key could not be parsed as base64, or the decoded key is invalid.
    InvalidSecretKeyData,

    /// Error in manifest file.
    InvalidManifest(&'static str),

    /// Signature in manifest could not be parsed as base64.
    InvalidSignatureData(base64::DecodeError),

    /// Signature verification failed.
    InvalidSignature,

    /// Digest verification of a (possibly newly) stored image failed.
    InvalidDigest,

    /// An operational error occurred.
    OperationError(&'static str),

    /// Curl failed in some way.
    DownloadError(String),

    /// Store failed because the version already exists.
    ///
    /// This can happen for two reasons:
    ///
    ///  * The version exists and has a different digest or size.
    ///  * Two versions differ only by separators, e.g. `1.0` and `1-0`.
    Duplicate(Version),

    /// There exists no version that matches the required bounds.
    ///
    /// E.g. we have 1.0, 1.1, and 1.2, but we require 2.*.
    NoCandidate,

    /// IO error.
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;

// TODO: Implement std::error::Error for Error.
