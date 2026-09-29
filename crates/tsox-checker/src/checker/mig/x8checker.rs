#![allow(dead_code, unused_imports, unused_variables)]

pub fn min_and_max<T>(slice: &[T], get_value: impl Fn(&T) -> i32) -> (i32, i32) {
    let mut min_value = 0;
    let mut max_value = 0;
    for (i, element) in slice.iter().enumerate() {
        let value = get_value(element);
        if i == 0 {
            min_value = value;
            max_value = value;
        } else {
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }
    (min_value, max_value)
}
