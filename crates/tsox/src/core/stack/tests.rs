use super::*;

#[test]
fn push_pop() {
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    assert_eq!(s.len(), 3);
    assert_eq!(s.pop(), 3);
    assert_eq!(s.pop(), 2);
    assert_eq!(s.pop(), 1);
    assert!(s.is_empty());
}

#[test]
fn peek() {
    let mut s = Stack::new();
    s.push("a");
    s.push("b");
    assert_eq!(*s.peek(), "b");
}
