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
        let left_num: u64 = left.parse().unwrap_or(0);
        let right_num: u64 = right.parse().unwrap_or(0);
        return left_num.cmp(&right_num);
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
    let text_lower = text.to_lowercase();
    let input: &str = &text_lower;

    // Parse major
    let (major_str, rest) = match input.find(|c: char| !c.is_ascii_digit()) {
        Some(0) => return Err(SemverParseError { input: text.to_string() }),
        Some(i) => (&input[..i], &input[i..]),
        None => (input, ""),
    };

    if !is_valid_numeric_component(major_str) {
        return Err(SemverParseError { input: text.to_string() });
    }
    let major: u32 = major_str.parse().map_err(|_| SemverParseError { input: text.to_string() })?;

    let mut minor = 0u32;
    let mut patch = 0u32;
    let mut prerelease = Vec::new();
    let mut build = Vec::new();
    let mut rest = rest;

    if rest.starts_with('.') {
        rest = &rest[1..];
        let (minor_str, rest2) = match rest.find(|c: char| !c.is_ascii_digit()) {
            Some(0) => return Err(SemverParseError { input: text.to_string() }),
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        if !is_valid_numeric_component(minor_str) {
            return Err(SemverParseError { input: text.to_string() });
        }
        minor = minor_str.parse().map_err(|_| SemverParseError { input: text.to_string() })?;
        rest = rest2;

        if rest.starts_with('.') {
            rest = &rest[1..];
            let (patch_str, rest3) = match rest.find(|c: char| c == '-' || c == '+') {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, ""),
            };
            if !is_valid_numeric_component(patch_str) {
                return Err(SemverParseError { input: text.to_string() });
            }
            patch = patch_str.parse().map_err(|_| SemverParseError { input: text.to_string() })?;
            rest = rest3;

            if rest.starts_with('-') {
                rest = &rest[1..];
                let (pre_str, rest4) = match rest.find('+') {
                    Some(i) => (&rest[..i], &rest[i..]),
                    None => (rest, ""),
                };
                if !is_valid_prerelease(pre_str) {
                    return Err(SemverParseError { input: text.to_string() });
                }
                prerelease = pre_str.split('.').map(|s| s.to_string()).collect();
                rest = rest4;
            }

            if rest.starts_with('+') {
                rest = &rest[1..];
                if !is_valid_build(rest) {
                    return Err(SemverParseError { input: text.to_string() });
                }
                build = rest.split('.').map(|s| s.to_string()).collect();
                rest = "";
            }
        }
    }

    if !rest.is_empty() {
        return Err(SemverParseError { input: text.to_string() });
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
            && (part == "0"
                || !part.starts_with('0')
                || part.chars().any(|c| !c.is_ascii_digit()))
    })
}

fn is_valid_build(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.').all(|part| {
        !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
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
        if self.alternatives.is_empty() {
            return write!(f, "*");
        }
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
            }
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
            (ComparatorOperator::LessThan, right.version.increment_major())
        } else if is_wildcard(&right.patch_str) {
            (ComparatorOperator::LessThan, right.version.increment_minor())
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
    let text = text.to_lowercase();
    let (prerelease_part, build_part) = split_prerelease_build(&text);

    let parts: Vec<&str> = prerelease_part.split('.').collect();
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

    let major = if is_wildcard(&major_str) { 0 } else { major_str.parse().ok()? };
    let minor = if is_wildcard(&minor_str) { 0 } else { minor_str.parse().ok()? };
    let patch = if is_wildcard(&patch_str) { 0 } else { patch_str.parse().ok()? };

    let prerelease = Vec::new();
    let build = if build_part.is_empty() {
        Vec::new()
    } else {
        build_part.split('.').map(|s| s.to_string()).collect()
    };

    Some(PartialVersion {
        version: Version { major, minor, patch, prerelease, build },
        major_str,
        minor_str,
        patch_str,
    })
}

fn split_prerelease_build(text: &str) -> (String, String) {
    // text is already lowercase, find prerelease and build parts
    // format: major.minor.patch-pre+build
    if let Some(plus_pos) = text.find('+') {
        let pre = text[..plus_pos].to_string();
        let build = text[plus_pos + 1..].to_string();
        (pre, build)
    } else {
        (text.to_string(), String::new())
    }
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
            comparators.push(VersionComparator { operator, operand: version });
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
            comparators.push(VersionComparator { operator, operand: version });
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
}
