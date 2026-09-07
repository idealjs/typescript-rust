#![allow(unused_imports)]

use super::*;

#[derive(Clone, Debug, Default)]
pub struct VersionRange {
    pub(crate) alternatives: Vec<Vec<VersionComparator>>,
}

#[derive(Clone, Debug)]
pub(crate) struct VersionComparator {
    pub(crate) operator: ComparatorOperator,
    pub(crate) operand: Version,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ComparatorOperator {
    LessThan,
    LessThanEqual,
    Equal,
    GreaterThanEqual,
    GreaterThan,
}

impl VersionRange {
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

pub fn try_parse_version_range(text: &str) -> Option<VersionRange> {
    parse_alternatives(text).map(|alts| VersionRange { alternatives: alts })
}

pub(crate) fn parse_alternatives(text: &str) -> Option<Vec<Vec<VersionComparator>>> {
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

pub(crate) fn parse_hyphen(text: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = text.splitn(3, ' ').collect();
    if parts.len() == 3 && parts[1] == "-" {
        return Some((parts[0].to_string(), parts[2].to_string()));
    }
    None
}

pub(crate) fn parse_hyphen_comparators(
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
pub(crate) struct PartialVersion {
    pub(crate) version: Version,
    pub(crate) major_str: String,
    pub(crate) minor_str: String,
    pub(crate) patch_str: String,
}

pub(crate) fn parse_partial(text: &str) -> Option<PartialVersion> {
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

pub(crate) fn split_partial(text: &str) -> (String, String, String) {
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

pub(crate) fn is_valid_partial_numeric(s: &str) -> bool {
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

pub(crate) fn parse_range_operator(text: &str) -> Option<(String, String)> {
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
