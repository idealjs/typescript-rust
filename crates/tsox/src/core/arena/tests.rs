use super::*;

#[test]
fn alloc_single() {
    let arena: Arena<i32> = Arena::new();
    let v = arena.alloc(42);
    assert_eq!(*v, 42);
}

#[test]
fn alloc_slice() {
    let arena: Arena<i32> = Arena::new();
    let s = arena.alloc_slice(3, |i| i as i32);
    assert_eq!(s, &[0, 1, 2]);
}

#[test]
fn clone_slice() {
    let arena: Arena<i32> = Arena::new();
    let original = [1, 2, 3, 4];
    let s = arena.clone_slice(&original);
    assert_eq!(s, &original[..]);
}
