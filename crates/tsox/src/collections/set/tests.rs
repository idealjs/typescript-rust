use super::*;

#[test]
fn basic() {
    let mut s = Set::new();
    s.add(1);
    s.add(2);
    s.add(1);
    assert_eq!(s.len(), 2);
    assert!(s.contains(&1));
    assert!(!s.contains(&3));
}

#[test]
fn union_and_subset() {
    let mut a = Set::from_items([1, 2, 3]);
    let b = Set::from_items([3, 4, 5]);
    a.union(&b);
    assert_eq!(a.len(), 5);
    assert!(Set::from_items([1, 2]).is_subset_of(&a));
}

#[test]
fn intersects() {
    let a = Set::from_items([1, 2, 3]);
    let b = Set::from_items([3, 4, 5]);
    let c = Set::from_items([4, 5, 6]);
    assert!(a.intersects(&b));
    assert!(!a.intersects(&c));
}
