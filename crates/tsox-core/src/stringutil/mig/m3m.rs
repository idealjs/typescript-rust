use super::unicode_data::unicode_esnext_identifier_part;
use super::unicode_data::unicode_esnext_identifier_start;

pub type Comparison = i32;

pub const COMPARISON_LESS_THAN: Comparison = -1;
pub const COMPARISON_EQUAL: Comparison = 0;
pub const COMPARISON_GREATER_THAN: Comparison = 1;

pub fn equate_string_case_insensitive(a: &str, b: &str) -> bool { crate::fntrace::enter("equate_string_case_insensitive"); 
    if a == b {
        return true;
    }
    let mut ai = a.char_indices().peekable();
    let mut bi = b.char_indices().peekable();
    loop {
        match (ai.next(), bi.next()) {
            (None, None) => return true,
            (None, Some(_)) | (Some(_), None) => return false,
            (Some((_, ca)), Some((_, cb))) => {
                if ca == cb {
                    continue;
                }
                let la = simple_fold_lower(ca);
                let lb = simple_fold_lower(cb);
                if la != lb {
                    return false;
                }
            }
        }
    }
}

fn simple_fold_lower(c: char) -> Vec<char> { crate::fntrace::enter("simple_fold_lower"); 
    c.to_lowercase().collect()
}

pub fn get_string_equality_comparer(ignore_case: bool) -> fn(&str, &str) -> bool { crate::fntrace::enter("get_string_equality_comparer"); 
    if ignore_case {
        equate_string_case_insensitive
    } else {
        equate_string_case_sensitive
    }
}

fn equate_string_case_sensitive(a: &str, b: &str) -> bool { crate::fntrace::enter("equate_string_case_sensitive"); 
    a == b
}

pub fn compare_strings_case_insensitive(a: &str, b: &str) -> Comparison { crate::fntrace::enter("compare_strings_case_insensitive"); 
    if a == b {
        return COMPARISON_EQUAL;
    }
    let mut ac = a.chars();
    let mut bc = b.chars();
    loop {
        match (ac.next(), bc.next()) {
            (None, None) => return COMPARISON_EQUAL,
            (None, Some(_)) => return COMPARISON_LESS_THAN,
            (Some(_), None) => return COMPARISON_GREATER_THAN,
            (Some(ca), Some(cb)) => {
                let lca: Vec<char> = ca.to_lowercase().collect();
                let lcb: Vec<char> = cb.to_lowercase().collect();
                if lca != lcb {
                    if lca < lcb {
                        return COMPARISON_LESS_THAN;
                    }
                    return COMPARISON_GREATER_THAN;
                }
            }
        }
    }
}

fn compare_strings_case_sensitive(a: &str, b: &str) -> Comparison { crate::fntrace::enter("compare_strings_case_sensitive"); 
    match a.cmp(b) {
        std::cmp::Ordering::Less => COMPARISON_LESS_THAN,
        std::cmp::Ordering::Equal => COMPARISON_EQUAL,
        std::cmp::Ordering::Greater => COMPARISON_GREATER_THAN,
    }
}

pub fn get_string_comparer(ignore_case: bool) -> fn(&str, &str) -> Comparison { crate::fntrace::enter("get_string_comparer"); 
    if ignore_case {
        compare_strings_case_insensitive
    } else {
        compare_strings_case_sensitive
    }
}

pub fn has_prefix(s: &str, prefix: &str, case_sensitive: bool) -> bool { crate::fntrace::enter("has_prefix"); 
    if case_sensitive {
        return s.starts_with(prefix);
    }
    if prefix.len() > s.len() {
        return false;
    }
    equate_string_case_insensitive(&s[..prefix.len()], prefix)
}

pub fn has_suffix(s: &str, suffix: &str, case_sensitive: bool) -> bool { crate::fntrace::enter("has_suffix"); 
    if case_sensitive {
        return s.ends_with(suffix);
    }
    if suffix.len() > s.len() {
        return false;
    }
    equate_string_case_insensitive(&s[s.len() - suffix.len()..], suffix)
}

pub fn has_prefix_and_suffix_without_overlap(
    s: &str,
    prefix: &str,
    suffix: &str,
    case_sensitive: bool,
) -> bool { crate::fntrace::enter("has_prefix_and_suffix_without_overlap"); 
    if prefix.len() + suffix.len() > s.len() {
        return false;
    }
    has_prefix(s, prefix, case_sensitive) && has_suffix(s, suffix, case_sensitive)
}

pub fn compare_strings_case_insensitive_then_sensitive(a: &str, b: &str) -> Comparison { crate::fntrace::enter("compare_strings_case_insensitive_then_sensitive"); 
    let cmp = compare_strings_case_insensitive(a, b);
    if cmp != COMPARISON_EQUAL {
        return cmp;
    }
    compare_strings_case_sensitive(a, b)
}

pub fn compare_strings_case_insensitive_eslint_compatible(a: &str, b: &str) -> Comparison { crate::fntrace::enter("compare_strings_case_insensitive_eslint_compatible"); 
    if a == b {
        return COMPARISON_EQUAL;
    }
    let a: String = a.chars().flat_map(char::to_lowercase).collect();
    let b: String = b.chars().flat_map(char::to_lowercase).collect();
    match a.cmp(&b) {
        std::cmp::Ordering::Less => COMPARISON_LESS_THAN,
        std::cmp::Ordering::Equal => COMPARISON_EQUAL,
        std::cmp::Ordering::Greater => COMPARISON_GREATER_THAN,
    }
}

pub fn is_unicode_identifier_start(ch: char) -> bool { crate::fntrace::enter("is_unicode_identifier_start"); 
    unicode_esnext_identifier_start(ch)
}

pub fn is_unicode_identifier_part(ch: char) -> bool { crate::fntrace::enter("is_unicode_identifier_part"); 
    unicode_esnext_identifier_part(ch)
}
