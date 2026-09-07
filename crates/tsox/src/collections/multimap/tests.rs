use super::*;

#[test]
fn add_and_get() {
    let mut m = MultiMap::new();
    m.add("a", 1);
    m.add("a", 2);
    m.add("b", 3);
    assert_eq!(m.get(&"a"), &[1, 2]);
    assert_eq!(m.get(&"b"), &[3]);
    assert_eq!(m.get(&"c"), &[] as &[i32]);
}

#[test]
fn remove() {
    let mut m = MultiMap::new();
    m.add("a", 1);
    m.add("a", 2);
    m.remove(&"a", &1);
    assert_eq!(m.get(&"a"), &[2]);
    m.remove(&"a", &2);
    assert!(!m.has(&"a"));
}

#[test]
fn group_by_works() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let m = group_by(&items, |x| x % 2);
    assert_eq!(m.get(&0), &[2, 4, 6]);
    assert_eq!(m.get(&1), &[1, 3, 5]);
}
