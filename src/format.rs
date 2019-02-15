// Tako -- Take container image.
// Copyright 2019 Ruud van Asseldonk.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Implementations of base64 formatting.
//!
//! See [RFC 3548](https://tools.ietf.org/html/rfc3548) for reference.
//! Especially the illustrations in section 7 are helpful when reading this
//! implementation.

const BASE64_CHARS: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
    'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f',
    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
    'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9', '+', '/',
];

/// String-format bytes as base64 (with + and /), append to the string.
pub fn append_base64(string: &mut String, bytes: &[u8]) {
    for triplet in bytes.chunks(3) {
        let len = triplet.len();
        let t: [u8; 3] = match len {
            1 => [triplet[0], 0, 0],
            2 => [triplet[0], triplet[1], 0],
            3 => [triplet[0], triplet[1], triplet[2]],
            _ => unreachable!(),
        };
        let i0 = t[0] >> 2;
        let i1 = (t[0] & 0b00_00_11) << 4 | (t[1] >> 4);
        let i2 = (t[1] & 0b00_11_11) << 2 | (t[2] >> 6);
        let i3 = t[2] & 0b11_11_11;
        string.push(BASE64_CHARS[i0 as usize]);
        string.push(BASE64_CHARS[i1 as usize]);
        string.push(if len > 1 { BASE64_CHARS[i2 as usize] } else { '=' });
        string.push(if len > 2 { BASE64_CHARS[i3 as usize] } else { '=' });
    }
}

/// String-format bytes as base64 (with + and /), append to the string.
pub fn encode_base64(bytes: &[u8]) -> String {
    let mut s = String::with_capacity((bytes.len() + 2) / 3 * 4);
    append_base64(&mut s, bytes);
    s
}

/// Return `i` such that `BASE64_CHARS[i] == ch`.
fn decode_base64_char(ch: u8) -> Option<u8> {
    match ch {
        _ if b'A' <= ch && ch <= b'Z' => Some(ch - b'A'),
        _ if b'a' <= ch && ch <= b'z' => Some(26 + (ch - b'a')),
        _ if b'0' <= ch && ch <= b'9' => Some(52 + (ch - b'0')),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decode a base64 (with + and /) string (encoded as UTF-8) back to bytes.
pub fn decode_base64<Bytes: AsRef<[u8]>>(b64: Bytes) -> Option<Vec<u8>> {
    // The input string length must be a multiple of 4.
    let max_bytes_len = match b64.as_ref().len() {
        n if n % 4 != 0 => return None,
        n => n / 4 * 3,
    };

    let mut bytes = Vec::with_capacity(max_bytes_len);

    for quartet in b64.as_ref().chunks(4) {
        let b0 = decode_base64_char(quartet[0])?;
        let b1 = decode_base64_char(quartet[1])?;
        bytes.push((b0 << 2) | (b1 >> 4));

        let b2 = match &quartet[2..4] {
            b"==" if bytes.len() == max_bytes_len - 2 => return Some(bytes),
            _ => decode_base64_char(quartet[2])?,
        };

        bytes.push((b1 & 0b00_11_11) << 4 | (b2 >> 2));

        let b3 = match quartet[3] {
            b'=' if bytes.len() == max_bytes_len - 1 => return Some(bytes),
            k => decode_base64_char(k)?,
        };

        bytes.push((b2 & 0b00_00_11) << 6 | b3);
    }

    Some(bytes)
}

#[cfg(test)]
mod test {
    use super::{append_base64, decode_base64};

    #[test]
    fn base64_slice_of_len_0_roundtrips() {
        let mut s = String::new();
        append_base64(&mut s, &[]);
        assert_eq!(s, "");
        assert_eq!(decode_base64("").unwrap(), &[]);
    }

    #[test]
    fn base64_slice_of_len_1_roundtrips() {
        for i in 0..256 {
            let data = [i as u8];
            let mut s = String::new();
            append_base64(&mut s, &data);
            assert_eq!(decode_base64(&s).unwrap(), &data);
        }
    }

    #[test]
    fn base64_slice_of_len_2_roundtrips() {
        for i in 0..256 {
            for j in 0..256 {
                let data = [i as u8, j as u8];
                let mut s = String::new();
                append_base64(&mut s, &data);
                assert_eq!(decode_base64(&s).unwrap(), &data);
            }
        }
    }

    #[test]
    fn base64_slice_of_len_3_roundtrips() {
        // Exhaustively testing all slices here slows down the tests too much.
        for &i in &[0, 1, 3, 254, 255] {
            for j in 0..256 {
                for k in 0..256 {
                    let data = [i as u8, j as u8, k as u8];
                    let mut s = String::new();
                    append_base64(&mut s, &data);
                    assert_eq!(decode_base64(&s).unwrap(), &data);
                }
            }
        }
    }

    #[test]
    fn base64_slice_of_len_4_roundtrips() {
        // Exhaustively testing all slices here slows down the tests too much.
        for &i in &[0, 255] {
            for &j in &[0, 1, 3, 254, 255] {
                for k in 0..256 {
                    for l in 0..256 {
                        let data = [i as u8, j as u8, k as u8, l as u8];
                        let mut s = String::new();
                        append_base64(&mut s, &data);
                        assert_eq!(decode_base64(&s).unwrap(), &data);
                    }
                }
            }
        }
    }

    #[test]
    fn base64_decode_fails_on_invalid_length() {
        assert!(decode_base64("a").is_none());
        assert!(decode_base64("ab").is_none());
        assert!(decode_base64("abc").is_none());
        assert!(decode_base64("abc_f").is_none());
        assert!(decode_base64("abc_fg").is_none());
        assert!(decode_base64("abc_fgh").is_none());
    }

    #[test]
    fn base64_decode_fails_on_invalid_characters() {
        assert!(decode_base64("abc\0").is_none());
        assert!(decode_base64("abc*").is_none());
        assert!(decode_base64("abc.").is_none());
        assert!(decode_base64("abc:").is_none());
        assert!(decode_base64("abc@").is_none());
        assert!(decode_base64("abc[").is_none());
        assert!(decode_base64("abc{").is_none());
    }

    #[test]
    fn base64_decode_fails_on_interior_padding() {
        assert!(decode_base64("=123").is_none());
        assert!(decode_base64("0=23").is_none());
        assert!(decode_base64("01=3").is_none());
        assert!(decode_base64("==23").is_none());
        assert!(decode_base64("0==3").is_none());
    }
}
