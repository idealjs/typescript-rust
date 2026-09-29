#![allow(dead_code, unused_imports, unused_variables)]

use std::cmp::Ordering;

use super::super::version::Version;
use super::super::version_range::{ComparatorOperator, VersionComparator};

pub fn test_comparator(comparator: &VersionComparator, version: &Version) -> bool {
    let cmp = version.compare(&comparator.operand);
    match comparator.operator {
        ComparatorOperator::LessThan => cmp == Ordering::Less,
        ComparatorOperator::LessThanEqual => cmp != Ordering::Greater,
        ComparatorOperator::Equal => cmp == Ordering::Equal,
        ComparatorOperator::GreaterThanEqual => cmp != Ordering::Less,
        ComparatorOperator::GreaterThan => cmp == Ordering::Greater,
    }
}

pub fn test_alternative(alternative: &[VersionComparator], version: &Version) -> bool {
    alternative.iter().all(|comparator| test_comparator(comparator, version))
}

pub fn test_disjunction(alternatives: &[Vec<VersionComparator>], version: &Version) -> bool {
    if alternatives.is_empty() {
        return true;
    }
    alternatives.iter().any(|alternative| test_alternative(alternative, version))
}
