use super::*;

#[test]
fn scope_restores_state() {
    let mut m = CopyOnWriteMap::new();
    m.insert("a", 1);
    m.insert("b", 2);
    m.with_scope(|m| {
        m.insert("c", 3);
        m.insert("a", 10);
        assert_eq!(m.get(&"a"), Some(&10));
        assert_eq!(m.len(), 3);
    });

    assert_eq!(m.get(&"a"), Some(&1));
    assert!(!m.contains_key(&"c"));
    assert_eq!(m.len(), 2);
}

#[test]
fn cow_set_scope() {
    let mut s = CopyOnWriteSet::new();
    s.insert("a");
    s.insert("b");
    s.with_scope(|s| {
        s.insert("c");
        assert!(s.contains(&"c"));
        assert_eq!(s.len(), 3);
    });
    assert!(!s.contains(&"c"));
    assert_eq!(s.len(), 2);
}

#[test]
fn explicit_enter_exit() {
    let mut m = CopyOnWriteMap::new();
    m.insert("a", 1);
    let state = m.enter_scope();
    m.insert("b", 2);
    assert_eq!(m.len(), 2);
    m.exit_scope(state);
    assert_eq!(m.len(), 1);
    assert!(!m.contains_key(&"b"));
}
