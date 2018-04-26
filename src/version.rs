// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Version parsing and ordering utilities.

use std::str::FromStr;

/// Designates a part of a version string.
#[derive(Debug, Eq, PartialEq)]
enum Part {
    /// A numeric part.
    Num(u64),

    /// A string (begin index and end index, inclusive and exclusive).
    ///
    /// We store two 32-bit integers rather than usizes, to ensure that this
    /// variant has the same size as `Num`. A version string is not larger than
    /// 4 GiB anyway, so this is fine.
    Str(u32, u32),
}

/// A parsed version string that can be ordered.
#[derive(Debug, Eq, PartialEq)]
struct Version {
    string: String,
    parts: Vec<Part>,
}

impl Version {
    fn push(parts: &mut Vec<Part>, string: &str, begin: usize, end: usize) {
        // Skip empty parts.
        if begin == end { return }

        let is_numeric = string
            .as_bytes()[begin..end]
            .iter()
            .all(|b| b.is_ascii_digit());

        if is_numeric {
            // The parse will not fail, as we just established that the string
            // consists of ascii digits only.
            // TODO: There might be an overflow issue though. Limit string
            // length as a crude solution?
            let n = u64::from_str(&string[begin..end]).unwrap();
            parts.push(Part::Num(n));
        } else {
            parts.push(Part::Str(begin as u32, end as u32))
        }
    }

    pub fn new(version: String) -> Version {
        let mut parts = Vec::new();
        let mut begin = 0;
        for (i, b) in version.as_bytes().iter().enumerate() {
            match *b {
                b'.' | b'-' | b'_' => {
                    // End the current part.
                    Version::push(&mut parts, &version, begin, i);
                    // Begin past the separator. The separator itself is
                    // not stored.
                    begin = i + 1;
                }
                _ => {},
            }
        }

        // Add the remaning part.
        Version::push(&mut parts, &version, begin, version.len());

        Version {
            string: version,
            parts: parts,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Part, Version};

    #[test]
    fn version_new_handles_empty() {
        let v = Version::new("".to_string());
        assert_eq!(v.parts.len(), 0);
    }

    #[test]
    fn version_new_handles_single_numeric_component() {
        let v = Version::new("13".to_string());
        assert_eq!(v.parts[0], Part::Num(13));
    }

    #[test]
    fn version_new_handles_single_string_component() {
        let v = Version::new("44cc".to_string());
        assert_eq!(v.parts[0], Part::Str(0, 4));
    }
}
