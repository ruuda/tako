// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Version parsing and ordering utilities.

use std::str::FromStr;
use std::cmp::Ordering;

/// A substring (begin index and end index, inclusive and exclusive).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Slice(u32, u32);

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
    Str(Slice),

    /// A part that sorts after all other parts, used to implement bounds.
    ///
    /// This part cannot be constructed by parsing a string, only by appending
    /// the part to an existing version.
    Inf,
}

/// A parsed version string that can be ordered.
///
/// Equality on versions is semantic equality, not string equality. The
/// following versions are all equal: `1.0.0`, `1_0_0`, and `1.0-0`. To compare
/// for string equality, use `as_str()`. Semantic equality does take the number
/// of parts into account. The following versions are not equal: `1`, `1.0`.
#[derive(Clone, Debug)]
pub struct Version {
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
            parts.push(Part::Str(Slice(begin as u32, end as u32)))
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

    /// Returns the slice of `Part::Str`.
    #[inline]
    fn part(&self, bounds: Slice) -> &str {
        &self.string[bounds.0 as usize..bounds.1 as usize]
    }

    /// Format the version as a string.
    pub fn as_str(&self) -> &str {
        &self.string[..]
    }

    /// Given a version pattern, return bounds (u, w) such that (u <= v < w).
    ///
    /// Examples:
    ///
    ///  * `1.0.* -> (1.0, 1.0.Inf)`
    ///  * `1.1.0 -> (1.1.0, 1.1.0)`
    pub fn pattern_to_bounds(mut self) -> (Version, Version) {
        let is_wildcard = match self.parts.last() {
            Some(&Part::Str(p)) => self.part(p) == "*",
            _ => false,
        };

        if !is_wildcard {
            (self.clone(), self)
        } else {
            self.parts.pop();
            let mut upper = self.clone();
            upper.parts.push(Part::Inf);
            (self, upper)
        }
    }
}

impl<'a> From<&'a str> for Version {
    fn from(v: &'a str) -> Version {
        Version::new(String::from(v))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        if self.parts.len() != other.parts.len() {
            return false
        }

        for (p, q) in self.parts.iter().zip(other.parts.iter()) {
            match (*p, *q) {
                (Part::Num(x), Part::Num(y)) if x == y => continue,
                (Part::Str(a), Part::Str(b)) if self.part(a) == other.part(b) => continue,
                (Part::Inf, Part::Inf) => continue,
                _ => return false,
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
                (Part::Str(a), Part::Str(b)) if self.part(a) == other.part(b) => continue,
                (Part::Str(a), Part::Str(b)) => return self.part(a).cmp(other.part(b)),
                // Inf sorts after anything apart from itself.
                (Part::Inf, Part::Inf) => continue,
                (_, Part::Inf) => return Ordering::Less,
                (Part::Inf, _) => return Ordering::Greater,
            }
        }

        // If all shared parts are equal, compare by number of parts (least
        // number of parts orders before most number of parts).
        self.parts.len().cmp(&other.parts.len())
    }
}

#[cfg(test)]
mod test {
    use super::{Part, Slice, Version};

    #[test]
    fn version_new_handles_empty() {
        let v = Version::from("");
        assert_eq!(v.parts.len(), 0);
    }

    #[test]
    fn version_new_handles_single_numeric_component() {
        let v = Version::from("13");
        assert_eq!(v.parts[0], Part::Num(13));
    }

    #[test]
    fn version_new_handles_single_string_component() {
        let v = Version::from("44cc");
        assert_eq!(v.parts[0], Part::Str(Slice(0, 4)));
    }

    #[test]
    fn version_new_handles_two_components() {
        let u = Version::from("1.0");
        let v = Version::from("1-0");
        let w = Version::from("1_0");
        assert_eq!(&u.parts, &[Part::Num(1), Part::Num(0)]);
        assert_eq!(&v.parts, &u.parts);
        assert_eq!(&w.parts, &u.parts);
    }

    #[test]
    fn version_eq_ignores_separator() {
        let u = Version::from("1.0");
        let v = Version::from("1-0");
        let w = Version::from("1_0");
        assert_eq!(u, v);
        assert_eq!(v, w);
    }

    #[test]
    fn version_eq_handles_pairwise_equal() {
        let versions = [
            Version::from("1.0.0"),
            Version::from("1_0.0"),
            Version::from("1.0-0"),
            Version::from("1.0.000"),
            Version::from("001.0.000"),
            Version::from("1.0.0."),
            Version::from("1.0.0____"),
            Version::from("1..0.0"),
            Version::from("1._.0.0"),
        ];
        for i in 0..versions.len() {
            for j in 0..versions.len() {
                assert_eq!(versions[i], versions[j]);
            }
        }
    }

    #[test]
    fn version_eq_handles_pairwise_inequal() {
        let versions = [
            Version::from("0"),
            Version::from("1"),
            Version::from("2"),
            Version::from("a"),
            Version::from("0.0"),
            Version::from("1.1"),
            Version::from("1.2"),
            Version::from("1.a"),
            Version::from("1.0"),
            Version::from("2.0"),
            Version::from("a.0"),
            Version::from("0.0.0"),
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
            Version::from(""),
            Version::from("a"),
            Version::from("a.b"),
            Version::from("a.0"),
            Version::from("a.0.0"),
            Version::from("a.1"),
            Version::from("b"),
            Version::from("b.0"),
            Version::from("b.1.3"),
            Version::from("c"),
            Version::from("0"),
            Version::from("0.a"),
            Version::from("0.0"),
            Version::from("0.1"),
            Version::from("0.1-a"),
            Version::from("0.1.1"),
            Version::from("1"),
            Version::from("1.0"),
            Version::from("1.0.1"),
            Version::from("1.1"),
            Version::from("2"),
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
