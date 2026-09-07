#![allow(unused_imports)]

use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Vec<String>,
    pub build: Vec<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
            prerelease: Vec::new(),
            build: Vec::new(),
        }
    }

    pub(crate) fn increment_major(&self) -> Version {
        Version::new(self.major + 1, 0, 0)
    }

    pub(crate) fn increment_minor(&self) -> Version {
        Version::new(self.major, self.minor + 1, 0)
    }

    pub(crate) fn increment_patch(&self) -> Version {
        Version::new(self.major, self.minor, self.patch + 1)
    }

    pub fn compare(&self, other: &Version) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            r => return r,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            r => return r,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            r => return r,
        }
        compare_prerelease(&self.prerelease, &other.prerelease)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.prerelease.is_empty() {
            write!(f, "-{}", self.prerelease.join("."))?;
        }
        if !self.build.is_empty() {
            write!(f, "+{}", self.build.join("."))?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

pub(crate) fn compare_prerelease(left: &[String], right: &[String]) -> Ordering {
    if left.is_empty() && right.is_empty() {
        return Ordering::Equal;
    }
    if left.is_empty() {
        return Ordering::Greater;
    }
    if right.is_empty() {
        return Ordering::Less;
    }
    for (l, r) in left.iter().zip(right.iter()) {
        let cmp = compare_prerelease_identifier(l, r);
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
    left.len().cmp(&right.len())
}

pub(crate) fn compare_prerelease_identifier(left: &str, right: &str) -> Ordering {
    let string_cmp = left.cmp(right);
    if string_cmp == Ordering::Equal {
        return Ordering::Equal;
    }

    let left_is_numeric = is_numeric_identifier(left);
    let right_is_numeric = is_numeric_identifier(right);

    if left_is_numeric || right_is_numeric {
        if !right_is_numeric {
            return Ordering::Less;
        }
        if !left_is_numeric {
            return Ordering::Greater;
        }
        return match (left.parse::<u64>(), right.parse::<u64>()) {
            (Ok(left_num), Ok(right_num)) => left_num.cmp(&right_num),

            _ => match left.len().cmp(&right.len()) {
                Ordering::Equal => string_cmp,
                len_cmp => len_cmp,
            },
        };
    }

    string_cmp
}

pub(crate) fn is_numeric_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) && (s == "0" || !s.starts_with('0'))
}

#[derive(Debug, Clone)]
pub struct SemverParseError {
    pub input: String,
}

impl fmt::Display for SemverParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not parse version string from {:?}", self.input)
    }
}

impl std::error::Error for SemverParseError {}

pub fn try_parse_version(text: &str) -> Result<Version, SemverParseError> {
    parse_version(text)
}

pub fn must_parse(text: &str) -> Version {
    try_parse_version(text).unwrap_or_else(|e| panic!("{}", e))
}

pub(crate) fn parse_version(text: &str) -> Result<Version, SemverParseError> {
    let input: &str = text;

    let (major_str, rest) = match input.find(|c: char| !c.is_ascii_digit()) {
        Some(0) => {
            return Err(SemverParseError {
                input: text.to_string(),
            });
        }
        Some(i) => (&input[..i], &input[i..]),
        None => (input, ""),
    };

    if !is_valid_numeric_component(major_str) {
        return Err(SemverParseError {
            input: text.to_string(),
        });
    }
    let major: u32 = major_str.parse().map_err(|_| SemverParseError {
        input: text.to_string(),
    })?;

    let mut minor = 0u32;
    let mut patch = 0u32;
    let mut prerelease = Vec::new();
    let mut build = Vec::new();
    let mut rest = rest;

    if rest.starts_with('.') {
        rest = &rest[1..];
        let (minor_str, rest2) = match rest.find(|c: char| !c.is_ascii_digit()) {
            Some(0) => {
                return Err(SemverParseError {
                    input: text.to_string(),
                });
            }
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        if !is_valid_numeric_component(minor_str) {
            return Err(SemverParseError {
                input: text.to_string(),
            });
        }
        minor = minor_str.parse().map_err(|_| SemverParseError {
            input: text.to_string(),
        })?;
        rest = rest2;

        if rest.starts_with('.') {
            rest = &rest[1..];
            let (patch_str, rest3) = match rest.find(|c: char| c == '-' || c == '+') {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, ""),
            };
            if !is_valid_numeric_component(patch_str) {
                return Err(SemverParseError {
                    input: text.to_string(),
                });
            }
            patch = patch_str.parse().map_err(|_| SemverParseError {
                input: text.to_string(),
            })?;
            rest = rest3;

            if rest.starts_with('-') {
                rest = &rest[1..];
                let (pre_str, rest4) = match rest.find('+') {
                    Some(i) => (&rest[..i], &rest[i..]),
                    None => (rest, ""),
                };
                if !is_valid_prerelease(pre_str) {
                    return Err(SemverParseError {
                        input: text.to_string(),
                    });
                }
                prerelease = pre_str.split('.').map(|s| s.to_string()).collect();
                rest = rest4;
            }

            if rest.starts_with('+') {
                rest = &rest[1..];
                if !is_valid_build(rest) {
                    return Err(SemverParseError {
                        input: text.to_string(),
                    });
                }
                build = rest.split('.').map(|s| s.to_string()).collect();
                rest = "";
            }
        }
    }

    if !rest.is_empty() {
        return Err(SemverParseError {
            input: text.to_string(),
        });
    }

    Ok(Version {
        major,
        minor,
        patch,
        prerelease,
        build,
    })
}

pub(crate) fn is_valid_numeric_component(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s == "0" {
        return true;
    }
    if s.starts_with('0') {
        return false;
    }
    s.chars().all(|c| c.is_ascii_digit())
}

pub(crate) fn is_valid_prerelease(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.').all(|part| {
        !part.is_empty()
            && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && (part == "0" || !part.starts_with('0') || part.chars().any(|c| !c.is_ascii_digit()))
    })
}

pub(crate) fn is_valid_build(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.')
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}
