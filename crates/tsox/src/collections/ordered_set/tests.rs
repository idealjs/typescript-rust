use super::*;

#[test]
fn insertion_order() {
    let mut s = OrderedSet::new();
    s.insert("b");
    s.insert("a");
    s.insert("b");
    s.insert("c");
    let values: Vec<&str> = s.iter().copied().collect();
    assert_eq!(values, vec!["b", "a", "c"]);
}

#[test]
fn remove() {
    let mut s = OrderedSet::new();
    s.insert("a");
    s.insert("b");
    s.insert("c");
    assert!(s.remove(&"b"));
    assert!(!s.remove(&"b"));
    let values: Vec<&str> = s.iter().copied().collect();
    assert_eq!(values, vec!["a", "c"]);
}

#[test]
fn test_ordered_set() {
    let mut s: OrderedSet<i32> = OrderedSet::new();

    s.add(1);
    s.add(2);
    s.add(3);

    assert!(s.has(&1));
    assert!(s.has(&2));
    assert!(s.has(&3));

    assert!(s.delete(&2));

    let values: Vec<i32> = s.iter().copied().collect();
    assert_eq!(values.len(), 2);
    assert!(values.windows(2).all(|w| w[0] <= w[1]));

    s.clear();

    assert_eq!(s.len(), 0);
    assert!(!s.has(&1));
    assert!(!s.has(&2));
    assert!(!s.has(&3));

    let s2 = s.clone();

    assert_eq!(s2.len(), 0);
}

#[test]
fn test_ordered_set_with_size_hint() {
    const N: usize = 1024;

    let mut s: OrderedSet<i32> = OrderedSet::with_capacity(N);
    for i in 0..N {
        s.add(i as i32);
    }

    assert_eq!(s.len(), N);
    for i in 0..N {
        assert!(
            s.has(&(i as i32)),
            "set pre-sized with with_capacity should contain {i}"
        );
    }

    let values: Vec<i32> = s.iter().copied().collect();
    assert_eq!(values.len(), N);
    for (idx, v) in values.iter().enumerate() {
        assert_eq!(*v, idx as i32, "insertion order broken at index {idx}");
    }

    s.add(0);
    assert_eq!(s.len(), N);
}
