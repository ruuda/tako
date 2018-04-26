// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Version parsing and ordering utilities.

use std::str::FromStr;
use std::cmp::Ordering;

/// Designates a part of a version string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
#[derive(Debug)]
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

    /// Returns the slice of `Part::Str`, or empty string for `Part::Num`.
    #[inline]
    fn part(&self, p: Part) -> &str {
        match p {
            Part::Num(..) => "",
            Part::Str(begin, end) => &self.string[begin as usize..end as usize],
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        if self.parts.len() != other.parts.len() {
            return false
        }

        for (p, q) in self.parts.iter().zip(other.parts.iter()) {
            match (*p, *q) {
                (Part::Num(..), Part::Str(..)) => return false,
                (Part::Str(..), Part::Num(..)) => return false,
                (Part::Num(x), Part::Num(y)) if x != y => return false,
                (Part::Num(_), Part::Num(_)) => continue,
                (str_a, str_b) if self.part(str_a) != other.part(str_b) => return false,
                _ => continue,
            }
        }

        true
    }
}

impl Eq for Version { }

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        for (p, q) in self.parts.iter().zip(other.parts.iter()) {
            match (*p, *q) {
                // Semi-arbitrary choice: string parts order before numeric
                // parts. This is because "1.0-a" feels like it should be before
                // "1.0.1". But really, just don't do that kind of thing ...
                (Part::Num(..), Part::Str(..)) => return Ordering::Greater,
                (Part::Str(..), Part::Num(..)) => return Ordering::Less,
                // Numeric parts order just by the number.
                (Part::Num(x), Part::Num(y)) if x == y => continue,
                (Part::Num(x), Part::Num(y)) => return x.cmp(&y),
                // String parts order lexicographically, ascending.
                (str_a, str_b) if self.part(str_a) == other.part(str_b) => continue,
                (str_a, str_b) => return self.part(str_a).cmp(other.part(str_b)),
            }
        }

        // If all shared parts are equal, compare by number of parts (least
        // number of parts orders before most number of parts).
        self.parts.len().cmp(&other.parts.len())
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

    #[test]
    fn version_new_handles_two_components() {
        let u = Version::new("1.0".to_string());
        let v = Version::new("1-0".to_string());
        let w = Version::new("1_0".to_string());
        assert_eq!(&u.parts, &[Part::Num(1), Part::Num(0)]);
        assert_eq!(&v.parts, &u.parts);
        assert_eq!(&w.parts, &u.parts);
    }

    #[test]
    fn version_eq_ignores_separator() {
        let u = Version::new("1.0".to_string());
        let v = Version::new("1-0".to_string());
        let w = Version::new("1_0".to_string());
        assert_eq!(u, v);
        assert_eq!(v, w);
    }

    #[test]
    fn version_eq_handles_pairwise_inequal() {
        let versions = [
            Version::new("0".to_string()),
            Version::new("1".to_string()),
            Version::new("2".to_string()),
            Version::new("a".to_string()),
            Version::new("0.0".to_string()),
            Version::new("1.1".to_string()),
            Version::new("1.2".to_string()),
            Version::new("1.a".to_string()),
            Version::new("1.0".to_string()),
            Version::new("2.0".to_string()),
            Version::new("a.0".to_string()),
            Version::new("0.0.0".to_string()),
        ];
        for i in 0..versions.len() {
            for j in 0..versions.len() {
                if i != j {
                    assert_ne!(versions[i], versions[j]);
                } else {
                    assert_eq!(versions[i], versions[j]);
                }
            }
        }
    }

    #[test]
    fn version_cmp_handles_pairwise_less() {
        // These versions are ordered in ascending order.
        let versions = [
            Version::new("".to_string()),
            Version::new("a".to_string()),
            Version::new("a.b".to_string()),
            Version::new("a.0".to_string()),
            Version::new("a.0.0".to_string()),
            Version::new("a.1".to_string()),
            Version::new("b".to_string()),
            Version::new("b.0".to_string()),
            Version::new("b.1.3".to_string()),
            Version::new("c".to_string()),
            Version::new("0".to_string()),
            Version::new("0.a".to_string()),
            Version::new("0.0".to_string()),
            Version::new("0.1".to_string()),
            Version::new("0.1-a".to_string()),
            Version::new("0.1.1".to_string()),
            Version::new("1".to_string()),
            Version::new("1.0".to_string()),
            Version::new("1.0.1".to_string()),
            Version::new("1.1".to_string()),
            Version::new("2".to_string()),
        ];
        for i in 0..versions.len() {
            for j in 0..versions.len() {
                let a = &versions[i];
                let b = &versions[j];
                assert_eq!(a.cmp(&b), i.cmp(&j), "{:?} vs {:?}", a, b);
            }
        }
    }
}
