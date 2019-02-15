// Tako -- Take container image.
// Copyright 2019 Ruud van Asseldonk.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

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

#[cfg(test)]
mod test {
    use base64;

    use super::{append_base64};

    #[test]
    fn base64_slice_of_len_1_roundtrips() {
        for i in 0..256 {
            let data = [i as u8];
            let mut s = String::new();
            append_base64(&mut s, &data);

            assert_eq!(base64::decode(&s).unwrap(), &data);
        }
    }

    #[test]
    fn base64_slice_of_len_2_roundtrips() {
        for i in 0..256 {
            for j in 0..256 {
                let data = [i as u8, j as u8];
                let mut s = String::new();
                append_base64(&mut s, &data);

                assert_eq!(base64::decode(&s).unwrap(), &data);
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

                    assert_eq!(base64::decode(&s).unwrap(), &data);
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

                        assert_eq!(base64::decode(&s).unwrap(), &data);
                    }
                }
            }
        }
    }
}
