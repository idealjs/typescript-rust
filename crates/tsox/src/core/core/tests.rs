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

#[test]
fn test_pattern_overlapping_match() {
    let p = try_parse_pattern("ab*ab");
    assert!(!p.matches("ab"), "'ab' should not match 'ab*ab'");
    assert!(p.matches("abXab"), "'abXab' should match 'ab*ab'");
    assert_eq!(p.matched_text("abXab"), "X");
    assert!(p.matches("abab"), "'abab' should match 'ab*ab'");
    assert_eq!(p.matched_text("abab"), "");
}
