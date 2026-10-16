pub(crate) use std::cmp::Ordering;

pub fn filter<T: Clone>(slice: &[T], f: impl Fn(&T) -> bool) -> Vec<T> { crate::fntrace::enter("filter"); 
    slice.iter().filter(|x| f(x)).cloned().collect()
}

pub fn map<T, U>(slice: &[T], f: impl Fn(&T) -> U) -> Vec<U> { crate::fntrace::enter("map"); 
    slice.iter().map(|x| f(x)).collect()
}

pub fn map_index<T, U>(slice: &[T], f: impl Fn(&T, usize) -> U) -> Vec<U> { crate::fntrace::enter("map_index"); 
    slice.iter().enumerate().map(|(i, x)| f(x, i)).collect()
}

pub fn map_filtered<T, U>(slice: &[T], f: impl Fn(&T) -> Option<U>) -> Vec<U> { crate::fntrace::enter("map_filtered"); 
    slice.iter().filter_map(|x| f(x)).collect()
}

pub fn flat_map<T, U: Clone>(slice: &[T], f: impl Fn(&T) -> &[U]) -> Vec<U> { crate::fntrace::enter("flat_map"); 
    slice.iter().flat_map(|x| f(x)).cloned().collect()
}

pub fn some<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool { crate::fntrace::enter("some"); 
    slice.iter().any(|x| f(x))
}

pub fn every<T>(slice: &[T], f: impl Fn(&T) -> bool) -> bool { crate::fntrace::enter("every"); 
    slice.iter().all(|x| f(x))
}

pub fn find<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> { crate::fntrace::enter("find"); 
    slice.iter().find(|x| f(x))
}

pub fn find_last<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<&T> { crate::fntrace::enter("find_last"); 
    slice.iter().rfind(|x| f(x))
}

pub fn find_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> { crate::fntrace::enter("find_index"); 
    slice.iter().position(|x| f(x))
}

pub fn find_last_index<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Option<usize> { crate::fntrace::enter("find_last_index"); 
    slice.iter().rposition(|x| f(x))
}

pub fn count_where<T>(slice: &[T], f: impl Fn(&T) -> bool) -> usize { crate::fntrace::enter("count_where"); 
    slice.iter().filter(|x| f(x)).count()
}

pub fn concatenate<T: Clone>(s1: &[T], s2: &[T]) -> Vec<T> { crate::fntrace::enter("concatenate"); 
    let mut v = Vec::with_capacity(s1.len() + s2.len());
    v.extend_from_slice(s1);
    v.extend_from_slice(s2);
    v
}

pub fn splice<T: Clone>(slice: &[T], start: isize, delete_count: usize, items: &[T]) -> Vec<T> { crate::fntrace::enter("splice"); 
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

pub fn replace_element<T: Clone>(slice: &[T], i: usize, t: T) -> Vec<T> { crate::fntrace::enter("replace_element"); 
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
) -> Vec<T> { crate::fntrace::enter("insert_sorted"); 
    let i = slice
        .binary_search_by(|probe| cmp(probe, element))
        .unwrap_or_else(|e| e);
    let mut result = Vec::with_capacity(slice.len() + 1);
    result.extend_from_slice(&slice[..i]);
    result.push(element.clone());
    result.extend_from_slice(&slice[i..]);
    result
}

pub fn min_all_func<T: Clone>(xs: &[T], cmp: impl Fn(&T, &T) -> Ordering) -> Vec<T> { crate::fntrace::enter("min_all_func"); 
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

pub fn append_if_unique<T: Clone + PartialEq>(slice: &[T], element: &T) -> Vec<T> { crate::fntrace::enter("append_if_unique"); 
    if slice.iter().any(|x| x == element) {
        return slice.to_vec();
    }
    let mut result = slice.to_vec();
    result.push(element.clone());
    result
}

pub fn memoize<T: Clone + Send + Sync + 'static>(
    create: impl FnOnce() -> T + Send + 'static,
) -> impl Fn() -> T { crate::fntrace::enter("memoize"); 
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

pub fn first_non_zero<T: Default + PartialEq + Clone>(values: &[T]) -> T { crate::fntrace::enter("first_non_zero"); 
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

pub fn try_parse_pattern(pattern: &str) -> Pattern { crate::fntrace::enter("try_parse_pattern"); 
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
    pub fn is_valid(&self) -> bool { crate::fntrace::enter("is_valid"); 
        self.star_index == -1 || self.star_index < self.text.len() as isize
    }

    pub fn matches(&self, candidate: &str) -> bool { crate::fntrace::enter("matches"); 
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

    pub fn matched_text<'a>(&self, candidate: &'a str) -> &'a str { crate::fntrace::enter("matched_text"); 
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
