// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Utilities for formatting, parsing, digests, etc.

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
