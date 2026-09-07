use super::*;

#[test]
fn basic() {
    let m = SyncMap::new();
    m.store("a", 1);
    m.store("b", 2);
    assert_eq!(m.load(&"a"), Some(1));
    assert_eq!(m.load(&"c"), None);
    assert_eq!(m.len(), 2);
}

#[test]
fn load_or_store() {
    let m = SyncMap::new();
    let (v, loaded) = m.load_or_store("a", 1);
    assert_eq!(v, 1);
    assert!(!loaded);
    let (v, loaded) = m.load_or_store("a", 2);
    assert_eq!(v, 1);
    assert!(loaded);
}

#[test]
fn test_sync_map_with_nil() {
    let m: SyncMap<String, Option<()>> = SyncMap::new();

    let got1 = m.load(&"foo".to_string());
    assert_eq!(got1, None);

    m.store("foo".to_string(), None);

    let got2 = m.load(&"foo".to_string());
    assert_eq!(got2, Some(None));

    let (too, loaded) = m.load_or_store("too".to_string(), None);
    assert!(!loaded);
    assert_eq!(too, None);

    m.for_each(|_, _| true);
}
