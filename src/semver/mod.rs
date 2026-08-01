//! Semantic versioning, ported from `internal/semver/`.
//!
//! Implements semver parsing, comparison, and version range matching
//! following the [semver.org](https://semver.org) specification and
//! [npm node-semver](https://github.com/npm/node-semver#range-grammar) range grammar.

use std::cmp::Ordering;
use std::fmt;

/// A semantic version.
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

    fn increment_major(&self) -> Version {
        Version::new(self.major + 1, 0, 0)
    }

    fn increment_minor(&self) -> Version {
        Version::new(self.major, self.minor + 1, 0)
    }

    fn increment_patch(&self) -> Version {
        Version::new(self.major, self.minor, self.patch + 1)
    }

    /// Compare two versions according to semver precedence rules.
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

fn compare_prerelease(left: &[String], right: &[String]) -> Ordering {
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

fn compare_prerelease_identifier(left: &str, right: &str) -> Ordering {
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
            // On overflow, compare by length, then fall back to string comparison.
            _ => match left.len().cmp(&right.len()) {
                Ordering::Equal => string_cmp,
                len_cmp => len_cmp,
            },
        };
    }

    string_cmp
}

fn is_numeric_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) && (s == "0" || !s.starts_with('0'))
}

/// Error returned when version parsing fails.
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

/// Try to parse a version string.
pub fn try_parse_version(text: &str) -> Result<Version, SemverParseError> {
    parse_version(text)
}

/// Parse a version string, panicking on failure.
pub fn must_parse(text: &str) -> Version {
    try_parse_version(text).unwrap_or_else(|e| panic!("{}", e))
}

fn parse_version(text: &str) -> Result<Version, SemverParseError> {
    let input: &str = text;

    // Parse major
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

fn is_valid_numeric_component(s: &str) -> bool {
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

fn is_valid_prerelease(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.').all(|part| {
        !part.is_empty()
            && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && (part == "0" || !part.starts_with('0') || part.chars().any(|c| !c.is_ascii_digit()))
    })
}

fn is_valid_build(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.')
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

// ─────────────────────────────────────────────────────────────────────
// Version ranges
// ─────────────────────────────────────────────────────────────────────

/// A version range, following npm's node-semver range grammar.
#[derive(Clone, Debug, Default)]
pub struct VersionRange {
    alternatives: Vec<Vec<VersionComparator>>,
}

#[derive(Clone, Debug)]
struct VersionComparator {
    operator: ComparatorOperator,
    operand: Version,
}

#[derive(Clone, Debug, PartialEq)]
enum ComparatorOperator {
    LessThan,
    LessThanEqual,
    Equal,
    GreaterThanEqual,
    GreaterThan,
}

impl VersionRange {
    /// Test whether a version satisfies this range.
    pub fn test(&self, version: &Version) -> bool {
        if self.alternatives.is_empty() {
            return true;
        }
        self.alternatives.iter().any(|alt| {
            alt.iter().all(|comp| {
                let cmp = version.compare(&comp.operand);
                match comp.operator {
                    ComparatorOperator::LessThan => cmp == Ordering::Less,
                    ComparatorOperator::LessThanEqual => cmp != Ordering::Greater,
                    ComparatorOperator::Equal => cmp == Ordering::Equal,
                    ComparatorOperator::GreaterThanEqual => cmp != Ordering::Less,
                    ComparatorOperator::GreaterThan => cmp == Ordering::Greater,
                }
            })
        })
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut empty = true;
        for (i, alt) in self.alternatives.iter().enumerate() {
            if i > 0 {
                write!(f, " || ")?;
            }
            for (j, comp) in alt.iter().enumerate() {
                if j > 0 {
                    write!(f, " ")?;
                }
                let op = match comp.operator {
                    ComparatorOperator::LessThan => "<",
                    ComparatorOperator::LessThanEqual => "<=",
                    ComparatorOperator::Equal => "=",
                    ComparatorOperator::GreaterThanEqual => ">=",
                    ComparatorOperator::GreaterThan => ">",
                };
                write!(f, "{}{}", op, comp.operand)?;
                empty = false;
            }
        }
        if empty {
            write!(f, "*")?;
        }
        Ok(())
    }
}

/// Try to parse a version range string.
pub fn try_parse_version_range(text: &str) -> Option<VersionRange> {
    parse_alternatives(text).map(|alts| VersionRange { alternatives: alts })
}

fn parse_alternatives(text: &str) -> Option<Vec<Vec<VersionComparator>>> {
    let text = text.trim();
    if text.is_empty() {
        return Some(Vec::new());
    }

    let mut alternatives = Vec::new();
    for range in text.split("||") {
        let range = range.trim();
        if range.is_empty() {
            continue;
        }

        // Check for hyphen range: "partial - partial"
        if let Some((left, right)) = parse_hyphen(range) {
            let (left_p, right_p) = (parse_partial(&left)?, parse_partial(&right)?);
            alternatives.push(parse_hyphen_comparators(&left_p, &right_p)?);
        } else {
            let mut comparators = Vec::new();
            for simple in range.split_whitespace() {
                let (op, ver) = parse_range_operator(simple.trim())?;
                let partial = parse_partial(&ver)?;
                comparators.extend(parse_comparator(&op, &partial)?);
            }
            alternatives.push(comparators);
        }
    }

    Some(alternatives)
}

fn parse_hyphen(text: &str) -> Option<(String, String)> {
    // Match "partial - partial"
    let parts: Vec<&str> = text.splitn(3, ' ').collect();
    if parts.len() == 3 && parts[1] == "-" {
        return Some((parts[0].to_string(), parts[2].to_string()));
    }
    None
}

fn parse_hyphen_comparators(
    left: &PartialVersion,
    right: &PartialVersion,
) -> Option<Vec<VersionComparator>> {
    let mut comparators = Vec::new();

    if !is_wildcard(&left.major_str) {
        comparators.push(VersionComparator {
            operator: ComparatorOperator::GreaterThanEqual,
            operand: left.version.clone(),
        });
    }

    if !is_wildcard(&right.major_str) {
        let (operator, operand) = if is_wildcard(&right.minor_str) {
            (
                ComparatorOperator::LessThan,
                right.version.increment_major(),
            )
        } else if is_wildcard(&right.patch_str) {
            (
                ComparatorOperator::LessThan,
                right.version.increment_minor(),
            )
        } else {
            (ComparatorOperator::LessThanEqual, right.version.clone())
        };
        comparators.push(VersionComparator { operator, operand });
    }

    Some(comparators)
}

#[derive(Clone, Debug)]
struct PartialVersion {
    version: Version,
    major_str: String,
    minor_str: String,
    patch_str: String,
}

fn parse_partial(text: &str) -> Option<PartialVersion> {
    let (core, prerelease_part, build_part) = split_partial(text);

    let parts: Vec<&str> = core.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let major_str = parts[0].to_string();
    if !is_wildcard(&major_str) && !is_valid_partial_numeric(&major_str) {
        return None;
    }

    let minor_str = if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "*".to_string()
    };
    if !is_wildcard(&minor_str) && !is_valid_partial_numeric(&minor_str) {
        return None;
    }

    let patch_str = if parts.len() > 2 {
        parts[2].to_string()
    } else {
        "*".to_string()
    };
    if !is_wildcard(&patch_str) && !is_valid_partial_numeric(&patch_str) {
        return None;
    }

    let major = if is_wildcard(&major_str) {
        0
    } else {
        major_str.parse().ok()?
    };
    let minor = if is_wildcard(&minor_str) {
        0
    } else {
        minor_str.parse().ok()?
    };
    let patch = if is_wildcard(&patch_str) {
        0
    } else {
        patch_str.parse().ok()?
    };

    let prerelease = if prerelease_part.is_empty() {
        Vec::new()
    } else {
        prerelease_part.split('.').map(|s| s.to_string()).collect()
    };
    let build = if build_part.is_empty() {
        Vec::new()
    } else {
        build_part.split('.').map(|s| s.to_string()).collect()
    };

    Some(PartialVersion {
        version: Version {
            major,
            minor,
            patch,
            prerelease,
            build,
        },
        major_str,
        minor_str,
        patch_str,
    })
}

fn split_partial(text: &str) -> (String, String, String) {
    // Split "major.minor.patch-pre+build" into (core, prerelease, build).
    let (before_build, build) = match text.find('+') {
        Some(pos) => (text[..pos].to_string(), text[pos + 1..].to_string()),
        None => (text.to_string(), String::new()),
    };
    let (core, prerelease) = match before_build.find('-') {
        Some(pos) => (
            before_build[..pos].to_string(),
            before_build[pos + 1..].to_string(),
        ),
        None => (before_build, String::new()),
    };
    (core, prerelease, build)
}

fn is_valid_partial_numeric(s: &str) -> bool {
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

fn parse_range_operator(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    if text.starts_with(">=") {
        return Some((">=".to_string(), text[2..].trim().to_string()));
    }
    if text.starts_with("<=") {
        return Some(("<=".to_string(), text[2..].trim().to_string()));
    }
    if text.starts_with('>') {
        return Some((">".to_string(), text[1..].trim().to_string()));
    }
    if text.starts_with('<') {
        return Some(("<".to_string(), text[1..].trim().to_string()));
    }
    if text.starts_with('=') {
        return Some(("=".to_string(), text[1..].trim().to_string()));
    }
    if text.starts_with('~') {
        return Some(("~".to_string(), text[1..].trim().to_string()));
    }
    if text.starts_with('^') {
        return Some(("^".to_string(), text[1..].trim().to_string()));
    }
    Some(("".to_string(), text.to_string()))
}

fn parse_comparator(op: &str, result: &PartialVersion) -> Option<Vec<VersionComparator>> {
    let operator_str = op;

    if is_wildcard(&result.major_str) {
        if op == "<" || op == ">" {
            return Some(vec![VersionComparator {
                operator: ComparatorOperator::LessThan,
                operand: Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    prerelease: vec!["0".to_string()],
                    build: Vec::new(),
                },
            }]);
        }
        return Some(Vec::new());
    }

    let mut comparators = Vec::new();

    match operator_str {
        "~" => {
            let first = VersionComparator {
                operator: ComparatorOperator::GreaterThanEqual,
                operand: result.version.clone(),
            };
            let second_version = if is_wildcard(&result.minor_str) {
                result.version.increment_major()
            } else {
                result.version.increment_minor()
            };
            let second = VersionComparator {
                operator: ComparatorOperator::LessThan,
                operand: second_version,
            };
            comparators.push(first);
            comparators.push(second);
        }
        "^" => {
            let first = VersionComparator {
                operator: ComparatorOperator::GreaterThanEqual,
                operand: result.version.clone(),
            };
            let second_version = if result.version.major > 0 || is_wildcard(&result.minor_str) {
                result.version.increment_major()
            } else if result.version.minor > 0 || is_wildcard(&result.patch_str) {
                result.version.increment_minor()
            } else {
                result.version.increment_patch()
            };
            let second = VersionComparator {
                operator: ComparatorOperator::LessThan,
                operand: second_version,
            };
            comparators.push(first);
            comparators.push(second);
        }
        "<" | ">=" => {
            let mut version = result.version.clone();
            if is_wildcard(&result.minor_str) || is_wildcard(&result.patch_str) {
                version.prerelease = vec!["0".to_string()];
            }
            let operator = if op == "<" {
                ComparatorOperator::LessThan
            } else {
                ComparatorOperator::GreaterThanEqual
            };
            comparators.push(VersionComparator {
                operator,
                operand: version,
            });
        }
        "<=" | ">" => {
            let mut version = result.version.clone();
            let operator;
            if is_wildcard(&result.minor_str) {
                operator = if op == "<=" {
                    ComparatorOperator::LessThan
                } else {
                    ComparatorOperator::GreaterThanEqual
                };
                version = version.increment_major();
                version.prerelease = vec!["0".to_string()];
            } else if is_wildcard(&result.patch_str) {
                operator = if op == "<=" {
                    ComparatorOperator::LessThan
                } else {
                    ComparatorOperator::GreaterThanEqual
                };
                version = version.increment_minor();
                version.prerelease = vec!["0".to_string()];
            } else {
                operator = if op == "<=" {
                    ComparatorOperator::LessThanEqual
                } else {
                    ComparatorOperator::GreaterThan
                };
            }
            comparators.push(VersionComparator {
                operator,
                operand: version,
            });
        }
        "=" | "" => {
            if is_wildcard(&result.minor_str) || is_wildcard(&result.patch_str) {
                let mut first_version = result.version.clone();
                first_version.prerelease = vec!["0".to_string()];
                let second_version = if is_wildcard(&result.minor_str) {
                    result.version.increment_major()
                } else {
                    result.version.increment_minor()
                };
                let mut second_version = second_version;
                second_version.prerelease = vec!["0".to_string()];
                comparators.push(VersionComparator {
                    operator: ComparatorOperator::GreaterThanEqual,
                    operand: first_version,
                });
                comparators.push(VersionComparator {
                    operator: ComparatorOperator::LessThan,
                    operand: second_version,
                });
            } else {
                comparators.push(VersionComparator {
                    operator: ComparatorOperator::Equal,
                    operand: result.version.clone(),
                });
            }
        }
        _ => return None,
    }

    Some(comparators)
}

fn is_wildcard(text: &str) -> bool {
    text == "*" || text == "x" || text == "X"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_version() {
        let v = try_parse_version("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.prerelease.is_empty());
        assert!(v.build.is_empty());
    }

    #[test]
    fn parse_prerelease() {
        let v = try_parse_version("1.0.0-alpha.1").unwrap();
        assert_eq!(v.prerelease, vec!["alpha", "1"]);
    }

    #[test]
    fn parse_build() {
        let v = try_parse_version("1.0.0+build.123").unwrap();
        assert_eq!(v.build, vec!["build", "123"]);
    }

    #[test]
    fn parse_partial() {
        let v = try_parse_version("1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn parse_invalid() {
        assert!(try_parse_version("1.2.3.4").is_err());
        assert!(try_parse_version("01.2.3").is_err());
        assert!(try_parse_version("abc").is_err());
    }

    #[test]
    fn version_compare() {
        let v1 = try_parse_version("1.0.0").unwrap();
        let v2 = try_parse_version("2.0.0").unwrap();
        let v3 = try_parse_version("1.0.0").unwrap();
        assert_eq!(v1.compare(&v2), Ordering::Less);
        assert_eq!(v2.compare(&v1), Ordering::Greater);
        assert_eq!(v1.compare(&v3), Ordering::Equal);
    }

    #[test]
    fn prerelease_compare() {
        let v1 = try_parse_version("1.0.0-alpha").unwrap();
        let v2 = try_parse_version("1.0.0").unwrap();
        assert_eq!(v1.compare(&v2), Ordering::Less);
    }

    #[test]
    fn version_display() {
        let v = try_parse_version("1.2.3-alpha.1+build.42").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.1+build.42");
    }

    #[test]
    fn range_exact() {
        let range = try_parse_version_range("1.2.3").unwrap();
        assert!(range.test(&try_parse_version("1.2.3").unwrap()));
        assert!(!range.test(&try_parse_version("1.2.4").unwrap()));
    }

    #[test]
    fn range_caret() {
        let range = try_parse_version_range("^1.2.3").unwrap();
        assert!(range.test(&try_parse_version("1.2.5").unwrap()));
        assert!(range.test(&try_parse_version("1.9.0").unwrap()));
        assert!(!range.test(&try_parse_version("2.0.0").unwrap()));
        assert!(!range.test(&try_parse_version("1.2.2").unwrap()));
    }

    #[test]
    fn range_tilde() {
        let range = try_parse_version_range("~1.2.3").unwrap();
        assert!(range.test(&try_parse_version("1.2.5").unwrap()));
        assert!(!range.test(&try_parse_version("1.3.0").unwrap()));
    }

    #[test]
    fn range_or() {
        let range = try_parse_version_range("1.0.0 || 2.0.0").unwrap();
        assert!(range.test(&try_parse_version("1.0.0").unwrap()));
        assert!(range.test(&try_parse_version("2.0.0").unwrap()));
        assert!(!range.test(&try_parse_version("3.0.0").unwrap()));
    }

    #[test]
    fn range_wildcard() {
        let range = try_parse_version_range("1.x").unwrap();
        assert!(range.test(&try_parse_version("1.0.0").unwrap()));
        assert!(range.test(&try_parse_version("1.5.3").unwrap()));
        assert!(!range.test(&try_parse_version("2.0.0").unwrap()));
    }

    // ─────────────────────────────────────────────────────────────────
    // Ported from Go internal/semver/version_test.go & version_range_test.go
    // ─────────────────────────────────────────────────────────────────

    use std::cmp::Ordering;

    // Mirrors Go's comparisonLessThan / comparisonEqualTo / comparisonGreaterThan.
    const LT: Ordering = Ordering::Less;
    const EQ: Ordering = Ordering::Equal;
    const GT: Ordering = Ordering::Greater;

    // Ported from TestTryParseSemver
    #[test]
    fn test_try_parse_semver() {
        let tests: &[(&str, Version)] = &[
            (
                "1.2.3-pre.4+build.5",
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec!["pre".into(), "4".into()],
                    build: vec!["build".into(), "5".into()],
                },
            ),
            (
                "1.2.3-pre.4",
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec!["pre".into(), "4".into()],
                    build: vec![],
                },
            ),
            (
                "1.2.3+build.4",
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec![],
                    build: vec!["build".into(), "4".into()],
                },
            ),
            (
                "1.2.3",
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec![],
                    build: vec![],
                },
            ),
        ];
        for (input, expected) in tests {
            let v = try_parse_version(input)
                .unwrap_or_else(|e| panic!("TryParseVersion({:?}): {}", input, e));
            assert_eq!(v.major, expected.major, "{}: major", input);
            assert_eq!(v.minor, expected.minor, "{}: minor", input);
            assert_eq!(v.patch, expected.patch, "{}: patch", input);
            assert_eq!(v.prerelease, expected.prerelease, "{}: prerelease", input);
            assert_eq!(v.build, expected.build, "{}: build", input);
        }
    }

    // Ported from TestVersionString
    #[test]
    fn test_version_string() {
        let tests: &[(Version, &str)] = &[
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec!["pre".into(), "4".into()],
                    build: vec!["build".into(), "5".into()],
                },
                "1.2.3-pre.4+build.5",
            ),
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec!["pre".into(), "4".into()],
                    build: vec!["build".into()],
                },
                "1.2.3-pre.4+build",
            ),
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec![],
                    build: vec!["build".into()],
                },
                "1.2.3+build",
            ),
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec!["pre".into(), "4".into()],
                    build: vec![],
                },
                "1.2.3-pre.4",
            ),
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec![],
                    build: vec!["build".into(), "4".into()],
                },
                "1.2.3+build.4",
            ),
            (
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: vec![],
                    build: vec![],
                },
                "1.2.3",
            ),
        ];
        for (version, expected) in tests {
            assert_eq!(version.to_string(), *expected);
        }
    }

    // Ported from TestVersionCompare
    #[test]
    fn test_version_compare() {
        let tests: &[(&str, &str, Ordering)] = &[
            // Major, minor, patch compared numerically
            ("1.0.0", "2.0.0", LT),
            ("1.0.0", "1.1.0", LT),
            ("1.0.0", "1.0.1", LT),
            ("2.0.0", "1.0.0", GT),
            ("1.1.0", "1.0.0", GT),
            ("1.0.1", "1.0.0", GT),
            ("1.0.0", "1.0.0", EQ),
            // Pre-release has lower precedence than normal
            ("1.0.0", "1.0.0-pre", GT),
            ("1.0.1-pre", "1.0.0", GT),
            ("1.0.0-pre", "1.0.0", LT),
            // Numeric identifiers compared numerically
            ("1.0.0-0", "1.0.0-1", LT),
            ("1.0.0-1", "1.0.0-0", GT),
            ("1.0.0-2", "1.0.0-10", LT),
            ("1.0.0-10", "1.0.0-2", GT),
            ("1.0.0-0", "1.0.0-0", EQ),
            // Letters/hyphens compared lexically in ASCII sort order
            ("1.0.0-a", "1.0.0-b", LT),
            ("1.0.0-a-2", "1.0.0-a-10", GT),
            ("1.0.0-b", "1.0.0-a", GT),
            ("1.0.0-a", "1.0.0-a", EQ),
            ("1.0.0-A", "1.0.0-a", LT),
            // Numeric always lower precedence than non-numeric
            ("1.0.0-0", "1.0.0-alpha", LT),
            ("1.0.0-alpha", "1.0.0-0", GT),
            ("1.0.0-0", "1.0.0-0", EQ),
            ("1.0.0-alpha", "1.0.0-alpha", EQ),
            // Larger set of pre-release fields has higher precedence
            ("1.0.0-alpha", "1.0.0-alpha.0", LT),
            ("1.0.0-alpha.0", "1.0.0-alpha", GT),
            // Compare dot-separated identifiers left to right
            ("1.0.0-a.0.b.1", "1.0.0-a.0.b.2", LT),
            ("1.0.0-a.0.b.1", "1.0.0-b.0.a.1", LT),
            ("1.0.0-a.0.b.2", "1.0.0-a.0.b.1", GT),
            ("1.0.0-b.0.a.1", "1.0.0-a.0.b.1", GT),
            // Build metadata does not figure into precedence
            ("1.0.0+build", "1.0.0", EQ),
            ("1.0.0+build.stuff", "1.0.0", EQ),
            ("1.0.0", "1.0.0+build", EQ),
            ("1.0.0+build", "1.0.0+stuff", EQ),
            // Edge cases for numeric and lexical comparison
            ("1.0.0-alpha.99999", "1.0.0-alpha.100000", LT),
            ("1.0.0-alpha.beta", "1.0.0-alpha.alpha", GT),
        ];
        for (v1s, v2s, want) in tests {
            let v1 = try_parse_version(v1s).unwrap();
            let v2 = try_parse_version(v2s).unwrap();
            assert_eq!(v1.compare(&v2), *want, "{} <=> {}", v1s, v2s);
        }
    }

    // Helper for TestVersionRanges
    fn assert_ranges_good_bad(version_range_string: &str, good: &[&str], bad: &[&str]) {
        let version_range = try_parse_version_range(version_range_string)
            .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", version_range_string));
        for g in good {
            let v = try_parse_version(g).unwrap();
            assert!(
                version_range.test(&v),
                "{} should be matched by range {}",
                g,
                version_range_string
            );
        }
        for b in bad {
            let v = try_parse_version(b).unwrap();
            assert!(
                !version_range.test(&v),
                "{} should not be matched by range {}",
                b,
                version_range_string
            );
        }
    }

    // Helper for comparator/conjunction/disjunction/hyphen/tilde/caret tests
    fn assert_range_test(name: &str, range_text: &str, version_text: &str, in_range: bool) {
        let version_range = try_parse_version_range(range_text)
            .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", range_text));
        let version = try_parse_version(version_text)
            .unwrap_or_else(|e| panic!("TryParseVersion({:?}): {}", version_text, e));
        assert_eq!(
            version_range.test(&version),
            in_range,
            "{}: version {} in range {} == {}",
            name,
            version_text,
            range_text,
            in_range
        );
    }

    // Ported from TestWildcardsHaveSameString
    #[test]
    fn test_wildcards_have_same_string() {
        fn assert_all_identical(name: &str, strs: &[&str]) {
            for &s1 in strs {
                for &s2 in strs {
                    let v1 = try_parse_version_range(s1)
                        .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", s1));
                    let v2 = try_parse_version_range(s2)
                        .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", s2));
                    assert_eq!(v1.to_string(), v2.to_string(), "{}: {} == {}", name, s1, s2);
                }
            }
        }
        let major = [
            "", "*", "*.*", "*.*.*", "x", "x.x", "x.x.x", "X", "X.X", "X.X.X",
        ];
        let minor = ["1", "1.*", "1.*.*", "1.x", "1.x.x", "1.X", "1.X.X"];
        let patch = ["1.2", "1.2.*", "1.2.x", "1.2.X"];
        let mixed = ["x", "X", "*", "x.X.x", "X.x.*"];
        assert_all_identical("majorWildcardStrings", &major);
        assert_all_identical("minorWildcardStrings", &minor);
        assert_all_identical("patchWildcardStrings", &patch);
        assert_all_identical("mixedCaseWildcardStrings", &mixed);
    }

    // Ported from TestVersionRanges
    #[test]
    fn test_version_ranges() {
        assert_ranges_good_bad(
            "1",
            &["1.0.0", "1.9.9", "1.0.0-pre", "1.0.0+build"],
            &["0.0.0", "2.0.0", "0.0.0-pre", "0.0.0+build"],
        );
        assert_ranges_good_bad(
            "1.2",
            &["1.2.0", "1.2.9", "1.2.0-pre", "1.2.0+build"],
            &["1.1.0", "1.3.0", "1.1.0-pre", "1.1.0+build"],
        );
        assert_ranges_good_bad(
            "1.2.3",
            &["1.2.3", "1.2.3+build"],
            &["1.2.2", "1.2.4", "1.2.2-pre", "1.2.2+build", "1.2.3-pre"],
        );
        assert_ranges_good_bad(
            "1.2.3-pre",
            &["1.2.3-pre", "1.2.3-pre+build.stuff"],
            &[
                "1.2.3",
                "1.2.3-pre.0",
                "1.2.3-pre.9",
                "1.2.3-pre.0+build",
                "1.2.3-pre.9+build",
                "1.2.3+build",
                "1.2.4",
            ],
        );
        assert_ranges_good_bad("<3.8.0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);
        assert_ranges_good_bad("<=3.8.0", &["3.6", "3.7", "3.8"], &["3.9", "4.0"]);
        assert_ranges_good_bad(">3.8.0", &["3.9", "4.0"], &["3.6", "3.7", "3.8"]);
        assert_ranges_good_bad(">=3.8.0", &["3.8", "3.9", "4.0"], &["3.6", "3.7"]);
        assert_ranges_good_bad("<3.8.0-0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);
        assert_ranges_good_bad("<=3.8.0-0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);

        // Big numbers in prerelease strings.
        let lotsa_ones = "1".repeat(320);
        let range_str = format!(">=1.2.3-1{}", lotsa_ones);
        let g0 = format!("1.2.3-1{}", lotsa_ones);
        let g1 = format!("1.2.3-11{}{}.1", lotsa_ones, "");
        let g2 = format!("1.2.3-1{}.1+build", lotsa_ones);
        let b0 = format!("1.2.3-{}.1+build", lotsa_ones);
        assert_ranges_good_bad(&range_str, &[&g0, &g1, &g2], &[&b0]);
    }

    // Ported from TestComparatorsOfVersionRanges
    #[test]
    fn test_comparators_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            // empty (matches everything)
            ("", "2.0.0", true),
            ("", "2.0.0-0", true),
            ("", "1.1.0", true),
            ("", "1.1.0-0", true),
            ("", "1.0.1", true),
            ("", "1.0.1-0", true),
            ("", "1.0.0", true),
            ("", "1.0.0-0", true),
            ("", "0.0.0", true),
            ("", "0.0.0-0", true),
            // wildcard major (matches everything)
            ("*", "2.0.0", true),
            ("*", "2.0.0-0", true),
            ("*", "1.1.0", true),
            ("*", "1.1.0-0", true),
            ("*", "1.0.1", true),
            ("*", "1.0.1-0", true),
            ("*", "1.0.0", true),
            ("*", "1.0.0-0", true),
            ("*", "0.0.0", true),
            ("*", "0.0.0-0", true),
            // wildcard minor
            ("1", "2.0.0", false),
            ("1", "2.0.0-0", false),
            ("1", "1.1.0", true),
            ("1", "1.1.0-0", true),
            ("1", "1.0.1", true),
            ("1", "1.0.1-0", true),
            ("1", "1.0.0", true),
            ("1", "1.0.0-0", true),
            ("1", "0.0.0", false),
            ("1", "0.0.0-0", false),
            // wildcard patch
            ("1.1", "2.0.0", false),
            ("1.1", "2.0.0-0", false),
            ("1.1", "1.1.0", true),
            ("1.1", "1.1.0-0", true),
            ("1.1", "1.0.1", false),
            ("1.1", "1.0.1-0", false),
            ("1.1", "1.0.0", false),
            ("1.1", "1.0.0-0", false),
            ("1.1", "0.0.0", false),
            ("1.1", "0.0.0-0", false),
            ("1.0", "2.0.0", false),
            ("1.0", "2.0.0-0", false),
            ("1.0", "1.1.0", false),
            ("1.0", "1.1.0-0", false),
            ("1.0", "1.0.1", true),
            ("1.0", "1.0.1-0", true),
            ("1.0", "1.0.0", true),
            ("1.0", "1.0.0-0", true),
            ("1.0", "0.0.0", false),
            ("1.0", "0.0.0-0", false),
            // exact
            ("1.1.0", "2.0.0", false),
            ("1.1.0", "2.0.0-0", false),
            ("1.1.0", "1.1.0", true),
            ("1.1.0", "1.1.0-0", false),
            ("1.1.0", "1.0.1", false),
            ("1.1.0", "1.0.1-0", false),
            ("1.1.0", "1.0.0-0", false),
            ("1.1.0", "1.0.0", false),
            ("1.1.0", "0.0.0", false),
            ("1.1.0", "0.0.0-0", false),
            ("1.1.0-0", "2.0.0", false),
            ("1.1.0-0", "2.0.0-0", false),
            ("1.1.0-0", "1.1.0", false),
            ("1.1.0-0", "1.1.0-0", true),
            ("1.1.0-0", "1.0.1", false),
            ("1.1.0-0", "1.0.1-0", false),
            ("1.1.0-0", "1.0.0-0", false),
            ("1.1.0-0", "1.0.0", false),
            ("1.1.0-0", "0.0.0", false),
            ("1.1.0-0", "0.0.0-0", false),
            ("1.0.1", "2.0.0", false),
            ("1.0.1", "2.0.0-0", false),
            ("1.0.1", "1.1.0", false),
            ("1.0.1", "1.1.0-0", false),
            ("1.0.1", "1.0.1", true),
            ("1.0.1", "1.0.1-0", false),
            ("1.0.1", "1.0.0-0", false),
            ("1.0.1", "1.0.0", false),
            ("1.0.1", "0.0.0", false),
            ("1.0.1", "0.0.0-0", false),
            ("1.0.1-0", "2.0.0", false),
            ("1.0.1-0", "2.0.0-0", false),
            ("1.0.1-0", "1.1.0", false),
            ("1.0.1-0", "1.1.0-0", false),
            ("1.0.1-0", "1.0.1", false),
            ("1.0.1-0", "1.0.1-0", true),
            ("1.0.1-0", "1.0.0-0", false),
            ("1.0.1-0", "1.0.0", false),
            ("1.0.1-0", "0.0.0", false),
            ("1.0.1-0", "0.0.0-0", false),
            ("1.0.0", "2.0.0", false),
            ("1.0.0", "2.0.0-0", false),
            ("1.0.0", "1.1.0", false),
            ("1.0.0", "1.1.0-0", false),
            ("1.0.0", "1.0.1", false),
            ("1.0.0", "1.0.1-0", false),
            ("1.0.0", "1.0.0-0", false),
            ("1.0.0", "1.0.0", true),
            ("1.0.0", "0.0.0", false),
            ("1.0.0", "0.0.0-0", false),
            ("1.0.0-0", "2.0.0", false),
            ("1.0.0-0", "2.0.0-0", false),
            ("1.0.0-0", "1.1.0", false),
            ("1.0.0-0", "1.1.0-0", false),
            ("1.0.0-0", "1.0.1", false),
            ("1.0.0-0", "1.0.1-0", false),
            ("1.0.0-0", "1.0.0", false),
            ("1.0.0-0", "1.0.0-0", true),
            // = wildcard major (matches everything)
            ("=*", "2.0.0", true),
            ("=*", "2.0.0-0", true),
            ("=*", "1.1.0", true),
            ("=*", "1.1.0-0", true),
            ("=*", "1.0.1", true),
            ("=*", "1.0.1-0", true),
            ("=*", "1.0.0", true),
            ("=*", "1.0.0-0", true),
            ("=*", "0.0.0", true),
            ("=*", "0.0.0-0", true),
            // = wildcard minor
            ("=1", "2.0.0", false),
            ("=1", "2.0.0-0", false),
            ("=1", "1.1.0", true),
            ("=1", "1.1.0-0", true),
            ("=1", "1.0.1", true),
            ("=1", "1.0.1-0", true),
            ("=1", "1.0.0", true),
            ("=1", "1.0.0-0", true),
            ("=1", "0.0.0", false),
            ("=1", "0.0.0-0", false),
            // = wildcard patch
            ("=1.1", "2.0.0", false),
            ("=1.1", "2.0.0-0", false),
            ("=1.1", "1.1.0", true),
            ("=1.1", "1.1.0-0", true),
            ("=1.1", "1.0.1", false),
            ("=1.1", "1.0.1-0", false),
            ("=1.1", "1.0.0", false),
            ("=1.1", "1.0.0-0", false),
            ("=1.1", "0.0.0", false),
            ("=1.1", "0.0.0-0", false),
            ("=1.0", "2.0.0", false),
            ("=1.0", "2.0.0-0", false),
            ("=1.0", "1.1.0", false),
            ("=1.0", "1.1.0-0", false),
            ("=1.0", "1.0.1", true),
            ("=1.0", "1.0.1-0", true),
            ("=1.0", "1.0.0", true),
            ("=1.0", "1.0.0-0", true),
            ("=1.0", "0.0.0", false),
            ("=1.0", "0.0.0-0", false),
            // = exact
            ("=1.1.0", "2.0.0", false),
            ("=1.1.0", "2.0.0-0", false),
            ("=1.1.0", "1.1.0", true),
            ("=1.1.0", "1.1.0-0", false),
            ("=1.1.0", "1.0.1", false),
            ("=1.1.0", "1.0.1-0", false),
            ("=1.1.0", "1.0.0-0", false),
            ("=1.1.0", "1.0.0", false),
            ("=1.1.0", "0.0.0", false),
            ("=1.1.0", "0.0.0-0", false),
            ("=1.1.0-0", "2.0.0", false),
            ("=1.1.0-0", "2.0.0-0", false),
            ("=1.1.0-0", "1.1.0", false),
            ("=1.1.0-0", "1.1.0-0", true),
            ("=1.1.0-0", "1.0.1", false),
            ("=1.1.0-0", "1.0.1-0", false),
            ("=1.1.0-0", "1.0.0-0", false),
            ("=1.1.0-0", "1.0.0", false),
            ("=1.1.0-0", "0.0.0", false),
            ("=1.1.0-0", "0.0.0-0", false),
            ("=1.0.1", "2.0.0", false),
            ("=1.0.1", "2.0.0-0", false),
            ("=1.0.1", "1.1.0", false),
            ("=1.0.1", "1.1.0-0", false),
            ("=1.0.1", "1.0.1", true),
            ("=1.0.1", "1.0.1-0", false),
            ("=1.0.1", "1.0.0-0", false),
            ("=1.0.1", "1.0.0", false),
            ("=1.0.1", "0.0.0", false),
            ("=1.0.1", "0.0.0-0", false),
            ("=1.0.1-0", "2.0.0", false),
            ("=1.0.1-0", "2.0.0-0", false),
            ("=1.0.1-0", "1.1.0", false),
            ("=1.0.1-0", "1.1.0-0", false),
            ("=1.0.1-0", "1.0.1", false),
            ("=1.0.1-0", "1.0.1-0", true),
            ("=1.0.1-0", "1.0.0-0", false),
            ("=1.0.1-0", "1.0.0", false),
            ("=1.0.1-0", "0.0.0", false),
            ("=1.0.1-0", "0.0.0-0", false),
            ("=1.0.0", "2.0.0", false),
            ("=1.0.0", "2.0.0-0", false),
            ("=1.0.0", "1.1.0", false),
            ("=1.0.0", "1.1.0-0", false),
            ("=1.0.0", "1.0.1", false),
            ("=1.0.0", "1.0.1-0", false),
            ("=1.0.0", "1.0.0-0", false),
            ("=1.0.0", "1.0.0", true),
            ("=1.0.0", "0.0.0", false),
            ("=1.0.0", "0.0.0-0", false),
            ("=1.0.0-0", "2.0.0", false),
            ("=1.0.0-0", "2.0.0-0", false),
            ("=1.0.0-0", "1.1.0", false),
            ("=1.0.0-0", "1.1.0-0", false),
            ("=1.0.0-0", "1.0.1", false),
            ("=1.0.0-0", "1.0.1-0", false),
            ("=1.0.0-0", "1.0.0", false),
            ("=1.0.0-0", "1.0.0-0", true),
            // > wildcard major (matches nothing)
            (">*", "2.0.0", false),
            (">*", "2.0.0-0", false),
            (">*", "1.1.0", false),
            (">*", "1.1.0-0", false),
            (">*", "1.0.1", false),
            (">*", "1.0.1-0", false),
            (">*", "1.0.0", false),
            (">*", "1.0.0-0", false),
            (">*", "0.0.0", false),
            (">*", "0.0.0-0", false),
            // > wildcard minor
            (">1", "2.0.0", true),
            (">1", "2.0.0-0", true),
            (">1", "1.1.0", false),
            (">1", "1.1.0-0", false),
            (">1", "1.0.1", false),
            (">1", "1.0.1-0", false),
            (">1", "1.0.0", false),
            (">1", "1.0.0-0", false),
            (">1", "0.0.0", false),
            (">1", "0.0.0-0", false),
            // > wildcard patch
            (">1.1", "2.0.0", true),
            (">1.1", "2.0.0-0", true),
            (">1.1", "1.1.0", false),
            (">1.1", "1.1.0-0", false),
            (">1.1", "1.0.1", false),
            (">1.1", "1.0.1-0", false),
            (">1.1", "1.0.0", false),
            (">1.1", "1.0.0-0", false),
            (">1.1", "0.0.0", false),
            (">1.1", "0.0.0-0", false),
            (">1.0", "2.0.0", true),
            (">1.0", "2.0.0-0", true),
            (">1.0", "1.1.0", true),
            (">1.0", "1.1.0-0", true),
            (">1.0", "1.0.1", false),
            (">1.0", "1.0.1-0", false),
            (">1.0", "1.0.0", false),
            (">1.0", "1.0.0-0", false),
            (">1.0", "0.0.0", false),
            (">1.0", "0.0.0-0", false),
            // > exact
            (">1.1.0", "2.0.0", true),
            (">1.1.0", "2.0.0-0", true),
            (">1.1.0", "1.1.0", false),
            (">1.1.0", "1.1.0-0", false),
            (">1.1.0", "1.0.1", false),
            (">1.1.0", "1.0.1-0", false),
            (">1.1.0", "1.0.0", false),
            (">1.1.0", "1.0.0-0", false),
            (">1.1.0", "0.0.0", false),
            (">1.1.0", "0.0.0-0", false),
            (">1.1.0-0", "2.0.0", true),
            (">1.1.0-0", "2.0.0-0", true),
            (">1.1.0-0", "1.1.0", true),
            (">1.1.0-0", "1.1.0-0", false),
            (">1.1.0-0", "1.0.1", false),
            (">1.1.0-0", "1.0.1-0", false),
            (">1.1.0-0", "1.0.0", false),
            (">1.1.0-0", "1.0.0-0", false),
            (">1.1.0-0", "0.0.0", false),
            (">1.1.0-0", "0.0.0-0", false),
            (">1.0.1", "2.0.0", true),
            (">1.0.1", "2.0.0-0", true),
            (">1.0.1", "1.1.0", true),
            (">1.0.1", "1.1.0-0", true),
            (">1.0.1", "1.0.1", false),
            (">1.0.1", "1.0.1-0", false),
            (">1.0.1", "1.0.0", false),
            (">1.0.1", "1.0.0-0", false),
            (">1.0.1", "0.0.0", false),
            (">1.0.1", "0.0.0-0", false),
            (">1.0.1-0", "2.0.0", true),
            (">1.0.1-0", "2.0.0-0", true),
            (">1.0.1-0", "1.1.0", true),
            (">1.0.1-0", "1.1.0-0", true),
            (">1.0.1-0", "1.0.1", true),
            (">1.0.1-0", "1.0.1-0", false),
            (">1.0.1-0", "1.0.0", false),
            (">1.0.1-0", "1.0.0-0", false),
            (">1.0.1-0", "0.0.0", false),
            (">1.0.1-0", "0.0.0-0", false),
            (">1.0.0", "2.0.0", true),
            (">1.0.0", "2.0.0-0", true),
            (">1.0.0", "1.1.0", true),
            (">1.0.0", "1.1.0-0", true),
            (">1.0.0", "1.0.1", true),
            (">1.0.0", "1.0.1-0", true),
            (">1.0.0", "1.0.0", false),
            (">1.0.0", "1.0.0-0", false),
            (">1.0.0", "0.0.0", false),
            (">1.0.0", "0.0.0-0", false),
            (">1.0.0-0", "2.0.0", true),
            (">1.0.0-0", "2.0.0-0", true),
            (">1.0.0-0", "1.1.0", true),
            (">1.0.0-0", "1.1.0-0", true),
            (">1.0.0-0", "1.0.1", true),
            (">1.0.0-0", "1.0.1-0", true),
            (">1.0.0-0", "1.0.0", true),
            (">1.0.0-0", "1.0.0-0", false),
            (">1.0.0-0", "0.0.0", false),
            (">1.0.0-0", "0.0.0-0", false),
            // >= wildcard major (matches everything)
            (">=*", "2.0.0", true),
            (">=*", "2.0.0-0", true),
            (">=*", "1.1.0", true),
            (">=*", "1.1.0-0", true),
            (">=*", "1.0.1", true),
            (">=*", "1.0.1-0", true),
            (">=*", "1.0.0", true),
            (">=*", "1.0.0-0", true),
            (">=*", "0.0.0", true),
            (">=*", "0.0.0-0", true),
            // >= wildcard minor
            (">=1", "2.0.0", true),
            (">=1", "2.0.0-0", true),
            (">=1", "1.1.0", true),
            (">=1", "1.1.0-0", true),
            (">=1", "1.0.1", true),
            (">=1", "1.0.1-0", true),
            (">=1", "1.0.0", true),
            (">=1", "1.0.0-0", true),
            (">=1", "0.0.0", false),
            (">=1", "0.0.0-0", false),
            // >= wildcard patch
            (">=1.1", "2.0.0", true),
            (">=1.1", "2.0.0-0", true),
            (">=1.1", "1.1.0", true),
            (">=1.1", "1.1.0-0", true),
            (">=1.1", "1.0.1", false),
            (">=1.1", "1.0.1-0", false),
            (">=1.1", "1.0.0", false),
            (">=1.1", "1.0.0-0", false),
            (">=1.1", "0.0.0", false),
            (">=1.1", "0.0.0-0", false),
            (">=1.0", "2.0.0", true),
            (">=1.0", "2.0.0-0", true),
            (">=1.0", "1.1.0", true),
            (">=1.0", "1.1.0-0", true),
            (">=1.0", "1.0.1", true),
            (">=1.0", "1.0.1-0", true),
            (">=1.0", "1.0.0", true),
            (">=1.0", "1.0.0-0", true),
            (">=1.0", "0.0.0", false),
            (">=1.0", "0.0.0-0", false),
            // >= exact
            (">=1.1.0", "2.0.0", true),
            (">=1.1.0", "2.0.0-0", true),
            (">=1.1.0", "1.1.0", true),
            (">=1.1.0", "1.1.0-0", false),
            (">=1.1.0", "1.0.1", false),
            (">=1.1.0", "1.0.1-0", false),
            (">=1.1.0", "1.0.0", false),
            (">=1.1.0", "1.0.0-0", false),
            (">=1.1.0", "0.0.0", false),
            (">=1.1.0", "0.0.0-0", false),
            (">=1.1.0-0", "2.0.0", true),
            (">=1.1.0-0", "2.0.0-0", true),
            (">=1.1.0-0", "1.1.0", true),
            (">=1.1.0-0", "1.1.0-0", true),
            (">=1.1.0-0", "1.0.1", false),
            (">=1.1.0-0", "1.0.1-0", false),
            (">=1.1.0-0", "1.0.0", false),
            (">=1.1.0-0", "1.0.0-0", false),
            (">=1.1.0-0", "0.0.0", false),
            (">=1.1.0-0", "0.0.0-0", false),
            (">=1.0.1", "2.0.0", true),
            (">=1.0.1", "2.0.0-0", true),
            (">=1.0.1", "1.1.0", true),
            (">=1.0.1", "1.1.0-0", true),
            (">=1.0.1", "1.0.1", true),
            (">=1.0.1", "1.0.1-0", false),
            (">=1.0.1", "1.0.0", false),
            (">=1.0.1", "1.0.0-0", false),
            (">=1.0.1", "0.0.0", false),
            (">=1.0.1", "0.0.0-0", false),
            (">=1.0.1-0", "2.0.0", true),
            (">=1.0.1-0", "2.0.0-0", true),
            (">=1.0.1-0", "1.1.0", true),
            (">=1.0.1-0", "1.1.0-0", true),
            (">=1.0.1-0", "1.0.1", true),
            (">=1.0.1-0", "1.0.1-0", true),
            (">=1.0.1-0", "1.0.0", false),
            (">=1.0.1-0", "1.0.0-0", false),
            (">=1.0.1-0", "0.0.0", false),
            (">=1.0.1-0", "0.0.0-0", false),
            (">=1.0.0", "2.0.0", true),
            (">=1.0.0", "2.0.0-0", true),
            (">=1.0.0", "1.1.0", true),
            (">=1.0.0", "1.1.0-0", true),
            (">=1.0.0", "1.0.1", true),
            (">=1.0.0", "1.0.1-0", true),
            (">=1.0.0", "1.0.0", true),
            (">=1.0.0", "1.0.0-0", false),
            (">=1.0.0", "0.0.0", false),
            (">=1.0.0", "0.0.0-0", false),
            (">=1.0.0-0", "2.0.0", true),
            (">=1.0.0-0", "2.0.0-0", true),
            (">=1.0.0-0", "1.1.0", true),
            (">=1.0.0-0", "1.1.0-0", true),
            (">=1.0.0-0", "1.0.1", true),
            (">=1.0.0-0", "1.0.1-0", true),
            (">=1.0.0-0", "1.0.0", true),
            (">=1.0.0-0", "1.0.0-0", true),
            (">=1.0.0-0", "0.0.0", false),
            (">=1.0.0-0", "0.0.0-0", false),
            // < wildcard major (matches nothing)
            ("<*", "2.0.0", false),
            ("<*", "2.0.0-0", false),
            ("<*", "1.1.0", false),
            ("<*", "1.1.0-0", false),
            ("<*", "1.0.1", false),
            ("<*", "1.0.1-0", false),
            ("<*", "1.0.0", false),
            ("<*", "1.0.0-0", false),
            ("<*", "0.0.0", false),
            ("<*", "0.0.0-0", false),
            // < wildcard minor
            ("<1", "2.0.0", false),
            ("<1", "2.0.0-0", false),
            ("<1", "1.1.0", false),
            ("<1", "1.1.0-0", false),
            ("<1", "1.0.1", false),
            ("<1", "1.0.1-0", false),
            ("<1", "1.0.0", false),
            ("<1", "1.0.0-0", false),
            ("<1", "0.0.0", true),
            ("<1", "0.0.0-0", true),
            // < wildcard patch
            ("<1.1", "2.0.0", false),
            ("<1.1", "2.0.0-0", false),
            ("<1.1", "1.1.0", false),
            ("<1.1", "1.1.0-0", false),
            ("<1.1", "1.0.1", true),
            ("<1.1", "1.0.1-0", true),
            ("<1.1", "1.0.0", true),
            ("<1.1", "1.0.0-0", true),
            ("<1.1", "0.0.0", true),
            ("<1.1", "0.0.0-0", true),
            ("<1.0", "2.0.0", false),
            ("<1.0", "2.0.0-0", false),
            ("<1.0", "1.1.0", false),
            ("<1.0", "1.1.0-0", false),
            ("<1.0", "1.0.1", false),
            ("<1.0", "1.0.1-0", false),
            ("<1.0", "1.0.0", false),
            ("<1.0", "1.0.0-0", false),
            ("<1.0", "0.0.0", true),
            ("<1.0", "0.0.0-0", true),
            // < exact
            ("<1.1.0", "2.0.0", false),
            ("<1.1.0", "2.0.0-0", false),
            ("<1.1.0", "1.1.0", false),
            ("<1.1.0", "1.1.0-0", true),
            ("<1.1.0", "1.0.1", true),
            ("<1.1.0", "1.0.1-0", true),
            ("<1.1.0", "1.0.0", true),
            ("<1.1.0", "1.0.0-0", true),
            ("<1.1.0", "0.0.0", true),
            ("<1.1.0", "0.0.0-0", true),
            ("<1.1.0-0", "2.0.0", false),
            ("<1.1.0-0", "2.0.0-0", false),
            ("<1.1.0-0", "1.1.0", false),
            ("<1.1.0-0", "1.1.0-0", false),
            ("<1.1.0-0", "1.0.1", true),
            ("<1.1.0-0", "1.0.1-0", true),
            ("<1.1.0-0", "1.0.0", true),
            ("<1.1.0-0", "1.0.0-0", true),
            ("<1.1.0-0", "0.0.0", true),
            ("<1.1.0-0", "0.0.0-0", true),
            ("<1.0.1", "2.0.0", false),
            ("<1.0.1", "2.0.0-0", false),
            ("<1.0.1", "1.1.0", false),
            ("<1.0.1", "1.1.0-0", false),
            ("<1.0.1", "1.0.1", false),
            ("<1.0.1", "1.0.1-0", true),
            ("<1.0.1", "1.0.0", true),
            ("<1.0.1", "1.0.0-0", true),
            ("<1.0.1", "0.0.0", true),
            ("<1.0.1", "0.0.0-0", true),
            ("<1.0.1-0", "2.0.0", false),
            ("<1.0.1-0", "2.0.0-0", false),
            ("<1.0.1-0", "1.1.0", false),
            ("<1.0.1-0", "1.1.0-0", false),
            ("<1.0.1-0", "1.0.1", false),
            ("<1.0.1-0", "1.0.1-0", false),
            ("<1.0.1-0", "1.0.0", true),
            ("<1.0.1-0", "1.0.0-0", true),
            ("<1.0.1-0", "0.0.0", true),
            ("<1.0.1-0", "0.0.0-0", true),
            ("<1.0.0", "2.0.0", false),
            ("<1.0.0", "2.0.0-0", false),
            ("<1.0.0", "1.1.0", false),
            ("<1.0.0", "1.1.0-0", false),
            ("<1.0.0", "1.0.1", false),
            ("<1.0.0", "1.0.1-0", false),
            ("<1.0.0", "1.0.0", false),
            ("<1.0.0", "1.0.0-0", true),
            ("<1.0.0", "0.0.0", true),
            ("<1.0.0", "0.0.0-0", true),
            ("<1.0.0-0", "2.0.0", false),
            ("<1.0.0-0", "2.0.0-0", false),
            ("<1.0.0-0", "1.1.0", false),
            ("<1.0.0-0", "1.1.0-0", false),
            ("<1.0.0-0", "1.0.1", false),
            ("<1.0.0-0", "1.0.1-0", false),
            ("<1.0.0-0", "1.0.0", false),
            ("<1.0.0-0", "1.0.0-0", false),
            ("<1.0.0-0", "0.0.0", true),
            ("<1.0.0-0", "0.0.0-0", true),
            // <= wildcard major (matches everything)
            ("<=*", "2.0.0", true),
            ("<=*", "2.0.0-0", true),
            ("<=*", "1.1.0", true),
            ("<=*", "1.1.0-0", true),
            ("<=*", "1.0.1", true),
            ("<=*", "1.0.1-0", true),
            ("<=*", "1.0.0", true),
            ("<=*", "1.0.0-0", true),
            ("<=*", "0.0.0", true),
            ("<=*", "0.0.0-0", true),
            // <= wildcard minor
            ("<=1", "2.0.0", false),
            ("<=1", "2.0.0-0", false),
            ("<=1", "1.1.0", true),
            ("<=1", "1.1.0-0", true),
            ("<=1", "1.0.1", true),
            ("<=1", "1.0.1-0", true),
            ("<=1", "1.0.0", true),
            ("<=1", "1.0.0-0", true),
            ("<=1", "0.0.0", true),
            ("<=1", "0.0.0-0", true),
            // <= wildcard patch
            ("<=1.1", "2.0.0", false),
            ("<=1.1", "2.0.0-0", false),
            ("<=1.1", "1.1.0", true),
            ("<=1.1", "1.1.0-0", true),
            ("<=1.1", "1.0.1", true),
            ("<=1.1", "1.0.1-0", true),
            ("<=1.1", "1.0.0", true),
            ("<=1.1", "1.0.0-0", true),
            ("<=1.1", "0.0.0", true),
            ("<=1.1", "0.0.0-0", true),
            ("<=1.0", "2.0.0", false),
            ("<=1.0", "2.0.0-0", false),
            ("<=1.0", "1.1.0", false),
            ("<=1.0", "1.1.0-0", false),
            ("<=1.0", "1.0.1", true),
            ("<=1.0", "1.0.1-0", true),
            ("<=1.0", "1.0.0", true),
            ("<=1.0", "1.0.0-0", true),
            ("<=1.0", "0.0.0", true),
            ("<=1.0", "0.0.0-0", true),
            // <= exact
            ("<=1.1.0", "2.0.0", false),
            ("<=1.1.0", "2.0.0-0", false),
            ("<=1.1.0", "1.1.0", true),
            ("<=1.1.0", "1.1.0-0", true),
            ("<=1.1.0", "1.0.1", true),
            ("<=1.1.0", "1.0.1-0", true),
            ("<=1.1.0", "1.0.0", true),
            ("<=1.1.0", "1.0.0-0", true),
            ("<=1.1.0", "0.0.0", true),
            ("<=1.1.0", "0.0.0-0", true),
            ("<=1.1.0-0", "2.0.0", false),
            ("<=1.1.0-0", "2.0.0-0", false),
            ("<=1.1.0-0", "1.1.0", false),
            ("<=1.1.0-0", "1.1.0-0", true),
            ("<=1.1.0-0", "1.0.1", true),
            ("<=1.1.0-0", "1.0.1-0", true),
            ("<=1.1.0-0", "1.0.0", true),
            ("<=1.1.0-0", "1.0.0-0", true),
            ("<=1.1.0-0", "0.0.0", true),
            ("<=1.1.0-0", "0.0.0-0", true),
            ("<=1.0.1", "2.0.0", false),
            ("<=1.0.1", "2.0.0-0", false),
            ("<=1.0.1", "1.1.0", false),
            ("<=1.0.1", "1.1.0-0", false),
            ("<=1.0.1", "1.0.1", true),
            ("<=1.0.1", "1.0.1-0", true),
            ("<=1.0.1", "1.0.0", true),
            ("<=1.0.1", "1.0.0-0", true),
            ("<=1.0.1", "0.0.0", true),
            ("<=1.0.1", "0.0.0-0", true),
            ("<=1.0.1-0", "2.0.0", false),
            ("<=1.0.1-0", "2.0.0-0", false),
            ("<=1.0.1-0", "1.1.0", false),
            ("<=1.0.1-0", "1.1.0-0", false),
            ("<=1.0.1-0", "1.0.1", false),
            ("<=1.0.1-0", "1.0.1-0", true),
            ("<=1.0.1-0", "1.0.0", true),
            ("<=1.0.1-0", "1.0.0-0", true),
            ("<=1.0.1-0", "0.0.0", true),
            ("<=1.0.1-0", "0.0.0-0", true),
            ("<=1.0.0", "2.0.0", false),
            ("<=1.0.0", "2.0.0-0", false),
            ("<=1.0.0", "1.1.0", false),
            ("<=1.0.0", "1.1.0-0", false),
            ("<=1.0.0", "1.0.1", false),
            ("<=1.0.0", "1.0.1-0", false),
            ("<=1.0.0", "1.0.0", true),
            ("<=1.0.0", "1.0.0-0", true),
            ("<=1.0.0", "0.0.0", true),
            ("<=1.0.0", "0.0.0-0", true),
            ("<=1.0.0-0", "2.0.0", false),
            ("<=1.0.0-0", "2.0.0-0", false),
            ("<=1.0.0-0", "1.1.0", false),
            ("<=1.0.0-0", "1.1.0-0", false),
            ("<=1.0.0-0", "1.0.1", false),
            ("<=1.0.0-0", "1.0.1-0", false),
            ("<=1.0.0-0", "1.0.0", false),
            ("<=1.0.0-0", "1.0.0-0", true),
            ("<=1.0.0-0", "0.0.0", true),
            ("<=1.0.0-0", "0.0.0-0", true),
            // https://github.com/microsoft/TypeScript/issues/50909
            (">4.8", "4.9.0-beta", true),
            (">=4.9", "4.9.0-beta", true),
            ("<4.9", "4.9.0-beta", false),
            ("<=4.8", "4.9.0-beta", false),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("comparators", range_text, version_text, *expected);
        }
    }

    // Ported from TestConjunctionsOfVersionRanges
    #[test]
    fn test_conjunctions_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            (">1.0.0 <2.0.0", "1.0.1", true),
            (">1.0.0 <2.0.0", "2.0.0", false),
            (">1.0.0 <2.0.0", "1.0.0", false),
            (">1 >2", "3.0.0", true),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("conjunctions", range_text, version_text, *expected);
        }
    }

    // Ported from TestDisjunctionsOfVersionRanges
    #[test]
    fn test_disjunctions_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            (">1.0.0 || <1.0.0", "1.0.1", true),
            (">1.0.0 || <1.0.0", "0.0.1", true),
            (">1.0.0 || <1.0.0", "1.0.0", false),
            (">1.0.0 || <1.0.0", "0.0.0", true),
            (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "1.0.0", true),
            (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "2.0.0", false),
            (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "3.0.0", true),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("disjunctions", range_text, version_text, *expected);
        }
    }

    // Ported from TestHyphensOfVersionRanges
    #[test]
    fn test_hyphens_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            ("1.0.0 - 2.0.0", "1.0.0", true),
            ("1.0.0 - 2.0.0", "1.0.1", true),
            ("1.0.0 - 2.0.0", "2.0.0", true),
            ("1.0.0 - 2.0.0", "2.0.1", false),
            ("1.0.0 - 2.0.0", "0.9.9", false),
            ("1.0.0 - 2.0.0", "3.0.0", false),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("hyphens", range_text, version_text, *expected);
        }
    }

    // Ported from TestTildesOfVersionRanges
    #[test]
    fn test_tildes_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            ("~0", "0.0.0", true),
            ("~0", "0.1.0", true),
            ("~0", "0.1.2", true),
            ("~0", "0.1.9", true),
            ("~0", "1.0.0", false),
            ("~0.1", "0.1.0", true),
            ("~0.1", "0.1.2", true),
            ("~0.1", "0.1.9", true),
            ("~0.1", "0.2.0", false),
            ("~0.1.2", "0.1.2", true),
            ("~0.1.2", "0.1.9", true),
            ("~0.1.2", "0.2.0", false),
            ("~1.0.0", "1.0.0", true),
            ("~1.0.0", "1.0.1", true),
            ("~1", "1.0.0", true),
            ("~1", "1.2.0", true),
            ("~1", "1.2.3", true),
            ("~1", "0.0.0", false),
            ("~1", "2.0.0", false),
            ("~1.2", "1.2.0", true),
            ("~1.2", "1.2.3", true),
            ("~1.2", "1.1.0", false),
            ("~1.2", "1.3.0", false),
            ("~1.2.3", "1.2.3", true),
            ("~1.2.3", "1.2.9", true),
            ("~1.2.3", "1.1.0", false),
            ("~1.2.3", "1.3.0", false),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("tilde", range_text, version_text, *expected);
        }
    }

    // Ported from TestCaretsOfVersionRanges
    #[test]
    fn test_carets_of_version_ranges() {
        let tests: &[(&str, &str, bool)] = &[
            ("^0", "0.0.0", true),
            ("^0", "0.1.0", true),
            ("^0", "0.9.0", true),
            ("^0", "0.1.2", true),
            ("^0", "0.1.9", true),
            ("^0", "1.0.0", false),
            ("^0.1", "0.1.0", true),
            ("^0.1", "0.1.2", true),
            ("^0.1", "0.1.9", true),
            ("^0.1.2", "0.1.2", true),
            ("^0.1.2", "0.1.9", true),
            ("^0.1.2", "0.0.0", false),
            ("^0.1.2", "0.2.0", false),
            ("^0.1.2", "1.0.0", false),
            ("^1", "1.0.0", true),
            ("^1", "1.2.0", true),
            ("^1", "1.2.3", true),
            ("^1", "1.9.0", true),
            ("^1", "0.0.0", false),
            ("^1", "2.0.0", false),
            ("^1.2", "1.2.0", true),
            ("^1.2", "1.2.3", true),
            ("^1.2", "1.9.0", true),
            ("^1.2", "1.1.0", false),
            ("^1.2", "2.0.0", false),
            ("^1.2.3", "1.2.3", true),
            ("^1.2.3", "1.9.0", true),
            ("^1.2.3", "1.2.2", false),
            ("^1.2.3", "2.0.0", false),
        ];
        for (range_text, version_text, expected) in tests {
            assert_range_test("caret", range_text, version_text, *expected);
        }
    }
}
