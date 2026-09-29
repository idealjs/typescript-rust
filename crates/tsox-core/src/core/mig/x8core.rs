#![allow(dead_code, unused_imports, unused_variables)]

pub fn last_or_nil<T: Clone + Default>(slice: &[T]) -> T {
    if !slice.is_empty() {
        return slice[slice.len() - 1].clone();
    }
    T::default()
}

pub fn map_non_nil<T, U: PartialEq + Default, F: FnMut(&T) -> U>(slice: &[T], mut f: F) -> Vec<U> {
    let mut result = Vec::new();
    for value in slice {
        let mapped = f(value);
        if mapped != U::default() {
            result.push(mapped);
        }
    }
    result
}
