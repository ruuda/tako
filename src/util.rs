// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Utilities for formatting, parsing, digests, files, etc.

use std::fs;
use std::io;
use std::path::Path;

use filebuffer::FileBuffer;
use sodiumoxide::crypto::hash::sha256;

use error::Result;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// String-format a bytes as lowercase hexadecimal, append to the string.
pub fn append_hex(string: &mut String, bytes: &[u8]) {
    for &b in bytes {
        string.push(HEX_CHARS[(b >> 4) as usize]);
        string.push(HEX_CHARS[(b & 0xf) as usize]);
    }
}

/// Compute the SHA256 digest of a file. Mmaps the file.
pub fn sha256sum(path: &Path) -> Result<sha256::Digest> {
    // Mmap the file when computing its digest. This way we can compute the
    // digest of files that don't fit in memory, without having to care about
    // streaming manually. Simple and fast.
    let fbuffer = FileBuffer::open(path)?;
    Ok(sha256::hash(&fbuffer))
}

/// A file that is deleted on drop, unless explicitly renamed.
///
/// This is used to write to a temporary file, which is cleaned up automatically
/// on an error: construct a `FileGuard` with the file path. In case of an early
/// return due to an error, the guard goes out of scope and deletes the file. If
/// the full write was successful, call `move_readonly()` to mark the file
/// read-only and move it into its final destination.
pub struct FileGuard<'a> {
    path: &'a Path,
    delete: bool,
}

impl<'a> FileGuard<'a> {
    pub fn new(path: &'a Path) -> FileGuard<'a> {
        FileGuard {
            path: path,
            delete: true,
        }
    }

    pub fn move_readonly(mut self, dest: &Path) -> io::Result<()> {
        // Make the file readonly.
        let mut perms = fs::metadata(self.path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(self.path, perms)?;
        fs::rename(self.path, dest)?;
        self.delete = false;
        Ok(())
    }
}

impl<'a> Drop for FileGuard<'a> {
    fn drop(&mut self) {
        if self.delete {
            // Remove the temp file. The drop with `delete` set happens on an
            // error path, so the file is likely incomplete, or its signature or
            // digest might be invalid. Removing the file is an operation that
            // may fail, but we are already in a failure mode, and deleting the
            // temp file is part of error recovery. If recovery fails, the
            // original error is more informative than the secondary IO error.
            // Besides, we cannot return the error here anyway. So ignore the
            // secondary error.
            let _ = fs::remove_file(self.path);
        }
    }
}
