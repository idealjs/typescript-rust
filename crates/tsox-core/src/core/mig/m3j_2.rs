use crate::core::compiler_options_options::CompilerOptions;
use crate::tspath::get_normalized_absolute_path::path_is_relative;
use crate::tspath::supported_ts_extensions_flat::{has_ts_file_extension, is_declaration_file_name};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

pub fn get_spelling_suggestion<T: Clone>(
    name: &str,
    candidates: impl Iterator<Item = T>,
    get_name: impl Fn(&T) -> String,
    compare: impl Fn(&T, &T) -> Ordering,
) -> Option<T> {
    get_spelling_suggestion_with_max_candidate_count(name, candidates, get_name, compare, 0)
}

pub fn get_spelling_suggestion_with_max_candidate_count<T: Clone>(
    name: &str,
    candidates: impl Iterator<Item = T>,
    get_name: impl Fn(&T) -> String,
    compare: impl Fn(&T, &T) -> Ordering,
    max_candidates: usize,
) -> Option<T> {
    let rune_name: Vec<char> = name.chars().collect();
    let maximum_length_difference = std::cmp::max(2, (rune_name.len() as f64 * 0.34) as usize);
    let mut best_distance = (rune_name.len() as f64 * 0.4).floor() + 0.9;
    let mut best_candidate: Option<T> = None;
    let mut checked_candidates = 0;
    for candidate in candidates {
        checked_candidates += 1;
        if max_candidates > 0 && checked_candidates > max_candidates {
            return None;
        }
        let candidate_name = get_name(&candidate);
        let max_len = std::cmp::max(candidate_name.chars().count(), rune_name.len());
        let min_len = std::cmp::min(candidate_name.chars().count(), rune_name.len());
        if !candidate_name.is_empty() && max_len - min_len <= maximum_length_difference {
            if candidate_name == name {
                continue;
            }
            if candidate_name.chars().count() < 3
                && !candidate_name.eq_ignore_ascii_case(name)
            {
                continue;
            }
            let candidate_runes: Vec<char> = candidate_name.chars().collect();
            let candidate_text: String = candidate_runes.iter().collect();
            let name_text: String = rune_name.iter().collect();
            let Some(distance) = levenshtein_with_max(&name_text, &candidate_text, best_distance)
            else {
                continue;
            };
            if distance < best_distance {
                best_distance = distance;
                best_candidate = Some(candidate);
            } else if best_candidate.is_none()
                || compare(&candidate, best_candidate.as_ref().unwrap()) == Ordering::Less
            {
                best_candidate = Some(candidate);
            }
        }
    }
    best_candidate
}

fn levenshtein_with_max(s1: &str, s2: &str, max: f64) -> Option<f64> {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let big = max + 0.01;
    let mut prev: Vec<f64> = (0..=s2.len()).map(|i| i as f64).collect();
    let mut curr = vec![0.0f64; s2.len() + 1];
    for i in 1..=s1.len() {
        let c1 = s1[i - 1];
        let min_j = (((i as f64) - max).ceil().max(1.0)) as usize;
        let max_j = ((max + i as f64).floor()) as usize;
        let max_j = max_j.min(s2.len());
        curr[0] = i as f64;
        let mut col_min = i as f64;
        for j in 1..(min_j.min(s2.len() + 1)) {
            curr[j] = big;
        }
        if min_j <= max_j {
            for j in min_j..=max_j {
                let substitution = if c1.to_lowercase().eq(s2[j - 1].to_lowercase()) {
                    prev[j - 1] + 0.1
                } else {
                    prev[j - 1] + 2.0
                };
                let dist = if c1 == s2[j - 1] {
                    prev[j - 1]
                } else {
                    (prev[j] + 1.0).min(curr[j - 1] + 1.0).min(substitution)
                };
                curr[j] = dist;
                col_min = col_min.min(dist);
            }
        }
        for j in (max_j + 1)..=s2.len() {
            curr[j] = big;
        }
        if col_min > max {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    let res = prev[s2.len()];
    if res > max {
        return None;
    }
    Some(res)
}

pub fn get_spelling_suggestion_for_strings(
    name: &str,
    candidates: impl Iterator<Item = String>,
) -> Option<String> {
    get_spelling_suggestion(
        name,
        candidates,
        |s: &String| s.clone(),
        |a: &String, b: &String| a.cmp(b),
    )
}

pub fn identity<T>(t: T) -> T {
    t
}

pub fn try_map<T, U, E>(
    slice: &[T],
    f: impl Fn(&T) -> Result<U, E>,
) -> Result<Vec<U>, E> {
    let mut result = Vec::with_capacity(slice.len());
    for value in slice {
        result.push(f(value)?);
    }
    Ok(result)
}

pub fn check_each_defined<'a, S>(s: &'a [Option<S>], msg: &str) -> Vec<&'a S> {
    s.iter().map(|v| v.as_ref().expect(msg)).collect()
}

pub fn index_after(s: &str, pattern: &str, start_index: usize) -> Option<usize> {
    if start_index > s.len() {
        return None;
    }
    s[start_index..]
        .find(pattern)
        .map(|matched| matched + start_index)
}

pub fn should_rewrite_module_specifier(
    specifier: &str,
    compiler_options: &CompilerOptions,
) -> bool {
    compiler_options
        .rewrite_relative_import_extensions
        .is_true()
        && path_is_relative(specifier)
        && !is_declaration_file_name(specifier)
        && has_ts_file_extension(specifier)
}

pub fn single_element_slice<T>(element: Option<&T>) -> Vec<&T> {
    match element {
        Some(element) => vec![element],
        None => Vec::new(),
    }
}

pub fn comparable_values_equal<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}

pub fn diff_maps<K: Eq + Hash + Clone, V: PartialEq + Clone>(
    m1: &HashMap<K, V>,
    m2: &HashMap<K, V>,
    on_added: &dyn Fn(&K, &V),
    on_removed: &dyn Fn(&K, &V),
    on_changed: &dyn Fn(&K, &V, &V),
) {
    diff_maps_func(m1, m2, &|a, b| a == b, on_added, on_removed, on_changed);
}

pub fn diff_maps_func<K, V1, V2>(
    m1: &HashMap<K, V1>,
    m2: &HashMap<K, V2>,
    equal_values: &dyn Fn(&V1, &V2) -> bool,
    on_added: &dyn Fn(&K, &V2),
    on_removed: &dyn Fn(&K, &V1),
    on_changed: &dyn Fn(&K, &V1, &V2),
) where
    K: Eq + Hash,
{
    for (k, v2) in m2 {
        if !m1.contains_key(k) {
            on_added(k, v2);
        }
    }
    for (k, v1) in m1 {
        match m2.get(k) {
            Some(v2) => {
                if !equal_values(v1, v2) {
                    on_changed(k, v1, v2);
                }
            }
            None => on_removed(k, v1),
        }
    }
}

pub fn unordered_equal<T: Eq + Hash>(s1: &[T], s2: &[T]) -> bool {
    if s1.len() != s2.len() {
        return false;
    }
    let mut counts: HashMap<&T, i64> = HashMap::new();
    for v in s1 {
        *counts.entry(v).or_insert(0) += 1;
    }
    for v in s2 {
        let count = counts.entry(v).or_insert(0);
        *count -= 1;
        if *count < 0 {
            return false;
        }
    }
    true
}

pub fn deduplicate<T: Clone + PartialEq>(slice: &[T]) -> Vec<T> {
    if slice.len() > 1 {
        for (i, value) in slice.iter().enumerate() {
            if slice[..i].contains(value) {
                let mut result = slice[..i].to_vec();
                for value in &slice[i + 1..] {
                    if !result.contains(value) {
                        result.push(value.clone());
                    }
                }
                return result;
            }
        }
    }
    slice.to_vec()
}

pub fn deduplicate_sorted<T: Clone>(slice: &[T], is_equal: impl Fn(&T, &T) -> bool) -> Vec<T> {
    if slice.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(slice.len());
    result.push(slice[0].clone());
    let mut last = &slice[0];
    for next in &slice[1..] {
        if is_equal(last, next) {
            continue;
        }
        result.push(next.clone());
        last = next;
    }
    result
}

pub fn compare_booleans(a: bool, b: bool) -> i32 {
    if a && !b {
        1
    } else if !a && b {
        -1
    } else {
        0
    }
}
