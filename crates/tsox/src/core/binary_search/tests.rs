use super::*;

#[test]
fn search_found() {
    let data = [1, 3, 5, 7, 9];
    let (i, found) = binary_search_unique_func(&data, |_, x| x.cmp(&5));
    assert!(found);
    assert_eq!(i, 2);
}

#[test]
fn search_not_found() {
    let data = [1, 3, 5, 7, 9];
    let (i, found) = binary_search_unique_func(&data, |_, x| x.cmp(&4));
    assert!(!found);
    assert_eq!(i, 2);
}

#[test]
fn search_empty() {
    let data: [i32; 0] = [];
    let (i, found) = binary_search_unique_func(&data, |_, _| Ordering::Equal);
    assert!(!found);
    assert_eq!(i, 0);
}
