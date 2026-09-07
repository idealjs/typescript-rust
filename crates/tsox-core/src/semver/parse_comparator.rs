#![allow(unused_imports)]

use super::*;

pub(crate) fn parse_comparator(
    op: &str,
    result: &PartialVersion,
) -> Option<Vec<VersionComparator>> {
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

pub(crate) fn is_wildcard(text: &str) -> bool {
    text == "*" || text == "x" || text == "X"
}
