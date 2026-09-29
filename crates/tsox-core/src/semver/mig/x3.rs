use crate::semver::version_range::{ComparatorOperator, VersionComparator};

pub(crate) fn format_disjunction(sb: &mut String, alternatives: &[Vec<VersionComparator>]) {
    let orig_len = sb.len();
    for (i, alternative) in alternatives.iter().enumerate() {
        if i > 0 {
            sb.push_str(" || ");
        }
        format_alternative(sb, alternative);
    }
    if sb.len() == orig_len {
        sb.push('*');
    }
}

pub(crate) fn format_alternative(sb: &mut String, comparators: &[VersionComparator]) {
    for (i, comparator) in comparators.iter().enumerate() {
        if i > 0 {
            sb.push(' ');
        }
        format_comparator(sb, comparator);
    }
}

pub(crate) fn format_comparator(sb: &mut String, comparator: &VersionComparator) {
    let operator = match comparator.operator {
        ComparatorOperator::LessThan => "<",
        ComparatorOperator::LessThanEqual => "<=",
        ComparatorOperator::Equal => "=",
        ComparatorOperator::GreaterThanEqual => ">=",
        ComparatorOperator::GreaterThan => ">",
    };
    sb.push_str(operator);
    sb.push_str(&comparator.operand.to_string());
}
