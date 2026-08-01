//! Generic slice/iterator helpers ported from `internal/core/core.go`.
//!
//! Many of these mirror functions that Go's standard library provides
//! (`slices.Filter`, `slices.Map`, etc.). In Rust we lean on iterators
//! and `Vec`, but we keep the named helpers so the rest of the compiler
//! can be ported with minimal churn.

use std::cmp::Ordering;

/// Apply a filter predicate to a slice, returning a new `Vec`.
///
/// Mirrors `core.Filter` in Go. If no elements are filtered out, the
/// input slice (cloned into a `Vec`) is returned as-is.
pub fn filter<T: Clone>(slice: &[T], f: impl Fn(&T) -> bool) -> Vec<T> {
    slice.iter().filter(|x| f(x)).cloned().collect()
}

/// Map every element of a slice through `f`.
///
/// Mirrors `core.Map` in Go.
pub fn map<T, U>(slice: &[T], f: impl Fn(&T) -> U) -> Vec<U> {
    slice.iter().map(|x| f(x)).collect()
}

/// Map with index.
pub fn map_index<T, U>(slice: &[T], f: impl Fn(&T, usize) -> U) -> Vec<U> {
    slice.iter().enumerate().map(|(i, x)| f(x, i)).collect()
}

/// Map and drop `None` results.
pub fn map_filtered<T, U>(slice: &[T], f: impl Fn(&T) -> Option<U>) -> Vec<U> {
    slice.iter().filter_map(|x| f(x)).collect()
}

/// Flat-map: apply `f` to each element and concatenate the results.
pub fn flat_map<T, U: Clone>(slice: &[T], f: impl Fn(&T) -> &[U]) -> Vec<U> {
    slice.iter().flat_map(|x| f(x)).cloned().collect()
}

/// True if any element satisfies `f`.
pub fn some<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool {
    slice.iter().any(|x| f(x))
}

/// True if every element satisfies `f`.
pub fn every<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool {
    slice.iter().all(|x| f(x))
}

/// Find the first element satisfying `f`, returning a reference.
pub fn find<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> {
    slice.iter().find(|x| f(x))
}

/// Find the last element satisfying `f`, returning a reference.
pub fn find_last<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> {
    slice.iter().rfind(|x| f(x))
}

/// Index of the first element satisfying `f`, or `None`.
pub fn find_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> {
    slice.iter().position(|x| f(x))
}

/// Index of the last element satisfying `f`, or `None`.
pub fn find_last_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> {
    slice.iter().rposition(|x| f(x))
}

/// Count elements satisfying `f`.
pub fn count_where<T>(slice: &[T], f: impl Fn(&T) -> bool) -> usize {
    slice.iter().filter(|x| f(x)).count()
}

/// Concatenate two slices into a new `Vec`.
pub fn concatenate<T: Clone>(s1: &[T], s2: &[T]) -> Vec<T> {
    let mut v = Vec::with_capacity(s1.len() + s2.len());
    v.extend_from_slice(s1);
    v.extend_from_slice(s2);
    v
}

/// Splice: remove `delete_count` items starting at `start` and insert `items`.
///
/// Mirrors `core.Splice` in Go (which mirrors JavaScript's `Array.prototype.splice`).
pub fn splice<T: Clone>(slice: &[T], start: isize, delete_count: usize, items: &[T]) -> Vec<T> {
    let len = slice.len() as isize;
    let mut start = start;
    if start < 0 {
        start = len + start;
    }
    if start < 0 {
        start = 0;
    }
    if start > len {
        start = len;
    }
    let start = start as usize;
    let end = (start + delete_count).min(slice.len());

    let mut result = Vec::with_capacity(slice.len() - (end - start) + items.len());
    result.extend_from_slice(&slice[..start]);
    result.extend_from_slice(items);
    result.extend_from_slice(&slice[end..]);
    result
}

/// Replace the element at index `i` with `t`, returning a new `Vec`.
pub fn replace_element<T: Clone>(slice: &[T], i: usize, t: T) -> Vec<T> {
    let mut result = slice.to_vec();
    if i < result.len() {
        result[i] = t;
    }
    result
}

/// Insert `element` into `slice` keeping it sorted according to `cmp`.
pub fn insert_sorted<T: Clone>(
    slice: &[T],
    element: &T,
    cmp: impl Fn(&T, &T) -> Ordering,
) -> Vec<T> {
    let i = slice
        .binary_search_by(|probe| cmp(probe, element))
        .unwrap_or_else(|e| e);
    let mut result = Vec::with_capacity(slice.len() + 1);
    result.extend_from_slice(&slice[..i]);
    result.push(element.clone());
    result.extend_from_slice(&slice[i..]);
    result
}

/// All minimum elements from `xs` according to `cmp`.
///
/// Mirrors `core.MinAllFunc` in Go.
pub fn min_all_func<T: Clone>(xs: &[T], cmp: impl Fn(&T, &T) -> Ordering) -> Vec<T> {
    if xs.is_empty() {
        return Vec::new();
    }
    let mut mins = vec![xs[0].clone()];
    for x in &xs[1..] {
        match cmp(x, &mins[0]) {
            Ordering::Less => {
                mins.clear();
                mins.push(x.clone());
            }
            Ordering::Equal => mins.push(x.clone()),
            Ordering::Greater => {}
        }
    }
    mins
}

/// Append `element` to `slice` if it is not already present (by equality).
pub fn append_if_unique<T: Clone + PartialEq>(slice: &[T], element: &T) -> Vec<T> {
    if slice.iter().any(|x| x == element) {
        return slice.to_vec();
    }
    let mut result = slice.to_vec();
    result.push(element.clone());
    result
}

/// Simple lazy memoization.
///
/// Mirrors `core.Memoize` in Go. The closure is called at most once; the
/// result is cached and reused on subsequent calls.
pub fn memoize<T: Clone + Send + Sync + 'static>(
    create: impl FnOnce() -> T + Send + 'static,
) -> impl Fn() -> T {
    let cell: std::sync::OnceLock<T> = std::sync::OnceLock::new();
    let create = std::sync::Mutex::new(Some(create));
    move || {
        cell.get_or_init(|| {
            let create = create
                .lock()
                .unwrap()
                .take()
                .expect("memoize closure called after init");
            create()
        })
        .clone()
    }
}

/// First non-zero value (by equality with the default).
pub fn first_non_zero<T: Default + PartialEq + Clone>(values: &[T]) -> T {
    let zero = T::default();
    for v in values {
        if v != &zero {
            return v.clone();
        }
    }
    zero
}

// ── Pattern matching, ported from `internal/core/pattern.go` ──

/// A glob-like pattern with at most one `*` wildcard.
///
/// Mirrors `core.Pattern` in Go. `star_index` is the byte offset of the single
/// `*` wildcard, or `-1` for an exact match (no wildcard).
#[derive(Debug, Clone, Default)]
pub struct Pattern {
    pub text: String,
    pub star_index: isize,
}

/// Parse a pattern string, mirroring `core.TryParsePattern` in Go.
///
/// Returns a `Pattern` with `star_index` set to the offset of the single `*`
/// wildcard, or `-1` if there is none. If the pattern contains more than one
/// `*`, returns an empty (invalid) pattern.
pub fn try_parse_pattern(pattern: &str) -> Pattern {
    match pattern.find('*') {
        None => Pattern {
            text: pattern.to_string(),
            star_index: -1,
        },
        Some(idx) => {
            if pattern[idx + 1..].contains('*') {
                Pattern::default() // more than one `*` → invalid
            } else {
                Pattern {
                    text: pattern.to_string(),
                    star_index: idx as isize,
                }
            }
        }
    }
}

impl Pattern {
    /// True if this pattern is valid (parsed from a string with at most one `*`).
    pub fn is_valid(&self) -> bool {
        self.star_index == -1 || self.star_index < self.text.len() as isize
    }

    /// True if `candidate` matches this pattern.
    pub fn matches(&self, candidate: &str) -> bool {
        if self.star_index == -1 {
            return self.text == candidate;
        }
        let idx = self.star_index as usize;
        let prefix = &self.text[..idx];
        let suffix = &self.text[idx + 1..];
        candidate.len() >= self.text.len() - 1
            && candidate.starts_with(prefix)
            && candidate.ends_with(suffix)
    }

    /// Return the text matched by the `*` wildcard. Panics if `candidate` does
    /// not match the pattern; returns an empty string for exact (no-wildcard)
    /// patterns.
    pub fn matched_text<'a>(&self, candidate: &'a str) -> &'a str {
        if !self.matches(candidate) {
            panic!("candidate does not match pattern");
        }
        if self.star_index == -1 {
            return "";
        }
        let idx = self.star_index as usize;
        let suffix_len = self.text.len() - idx - 1;
        &candidate[idx..candidate.len() - suffix_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_works() {
        assert_eq!(filter(&[1, 2, 3, 4], |x| x % 2 == 0), vec![2, 4]);
    }

    #[test]
    fn map_works() {
        assert_eq!(map(&[1, 2, 3], |x| x * 2), vec![2, 4, 6]);
    }

    #[test]
    fn splice_works() {
        assert_eq!(splice(&[1, 2, 3, 4], 1, 1, &[9]), vec![1, 9, 3, 4]);
        assert_eq!(splice(&[1, 2, 3], -1, 1, &[]), vec![1, 2]);
    }

    #[test]
    fn insert_sorted_works() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(insert_sorted(&[1, 3, 5], &4, cmp), vec![1, 3, 4, 5]);
    }

    #[test]
    fn append_if_unique_works() {
        assert_eq!(append_if_unique(&[1, 2], &2), vec![1, 2]);
        assert_eq!(append_if_unique(&[1, 2], &3), vec![1, 2, 3]);
    }

    // ── Ported from Go internal/core/pattern_test.go ──

    #[test]
    fn test_pattern_overlapping_match() {
        // Ported from TestPatternOverlappingMatch.
        let p = try_parse_pattern("ab*ab");
        assert!(!p.matches("ab"), "'ab' should not match 'ab*ab'");
        assert!(p.matches("abXab"), "'abXab' should match 'ab*ab'");
        assert_eq!(p.matched_text("abXab"), "X");
        assert!(p.matches("abab"), "'abab' should match 'ab*ab'");
        assert_eq!(p.matched_text("abab"), "");
    }
}
