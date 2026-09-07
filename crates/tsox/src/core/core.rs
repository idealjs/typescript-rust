use std::cmp::Ordering;

pub fn filter<T: Clone>(slice: &[T], f: impl Fn(&T) -> bool) -> Vec<T> {
    slice.iter().filter(|x| f(x)).cloned().collect()
}

pub fn map<T, U>(slice: &[T], f: impl Fn(&T) -> U) -> Vec<U> {
    slice.iter().map(|x| f(x)).collect()
}

pub fn map_index<T, U>(slice: &[T], f: impl Fn(&T, usize) -> U) -> Vec<U> {
    slice.iter().enumerate().map(|(i, x)| f(x, i)).collect()
}

pub fn map_filtered<T, U>(slice: &[T], f: impl Fn(&T) -> Option<U>) -> Vec<U> {
    slice.iter().filter_map(|x| f(x)).collect()
}

pub fn flat_map<T, U: Clone>(slice: &[T], f: impl Fn(&T) -> &[U]) -> Vec<U> {
    slice.iter().flat_map(|x| f(x)).cloned().collect()
}

pub fn some<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool {
    slice.iter().any(|x| f(x))
}

pub fn every<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool {
    slice.iter().all(|x| f(x))
}

pub fn find<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> {
    slice.iter().find(|x| f(x))
}

pub fn find_last<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> {
    slice.iter().rfind(|x| f(x))
}

pub fn find_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> {
    slice.iter().position(|x| f(x))
}

pub fn find_last_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> {
    slice.iter().rposition(|x| f(x))
}

pub fn count_where<T>(slice: &[T], f: impl Fn(&T) -> bool) -> usize {
    slice.iter().filter(|x| f(x)).count()
}

pub fn concatenate<T: Clone>(s1: &[T], s2: &[T]) -> Vec<T> {
    let mut v = Vec::with_capacity(s1.len() + s2.len());
    v.extend_from_slice(s1);
    v.extend_from_slice(s2);
    v
}

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

pub fn replace_element<T: Clone>(slice: &[T], i: usize, t: T) -> Vec<T> {
    let mut result = slice.to_vec();
    if i < result.len() {
        result[i] = t;
    }
    result
}

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

pub fn append_if_unique<T: Clone + PartialEq>(slice: &[T], element: &T) -> Vec<T> {
    if slice.iter().any(|x| x == element) {
        return slice.to_vec();
    }
    let mut result = slice.to_vec();
    result.push(element.clone());
    result
}

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

pub fn first_non_zero<T: Default + PartialEq + Clone>(values: &[T]) -> T {
    let zero = T::default();
    for v in values {
        if v != &zero {
            return v.clone();
        }
    }
    zero
}

#[derive(Debug, Clone, Default)]
pub struct Pattern {
    pub text: String,
    pub star_index: isize,
}

pub fn try_parse_pattern(pattern: &str) -> Pattern {
    match pattern.find('*') {
        None => Pattern {
            text: pattern.to_string(),
            star_index: -1,
        },
        Some(idx) => {
            if pattern[idx + 1..].contains('*') {
                Pattern::default()
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
    pub fn is_valid(&self) -> bool {
        self.star_index == -1 || self.star_index < self.text.len() as isize
    }

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
mod tests;
