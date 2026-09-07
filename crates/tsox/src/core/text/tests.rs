use super::*;

#[test]
fn range_basics() {
    let r = TextRange::new(5, 10);
    assert_eq!(r.pos(), 5);
    assert_eq!(r.end(), 10);
    assert_eq!(r.len(), 5);
    assert!(r.contains(5));
    assert!(r.contains(9));
    assert!(!r.contains(10));
    assert!(r.contains_inclusive(10));
}

#[test]
fn range_overlap() {
    let a = TextRange::new(0, 5);
    let b = TextRange::new(5, 10);
    assert!(!a.overlaps(&b));
    assert!(a.intersects(&b));
}
