use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn marshal_and_unmarshal() {
    let p = Point { x: 1, y: 2 };
    let json = marshal(&p).unwrap();
    assert_eq!(json, r#"{"x":1,"y":2}"#);
    let p2: Point = unmarshal(&json).unwrap();
    assert_eq!(p, p2);
}

#[test]
fn marshal_indent_works() {
    let p = Point { x: 1, y: 2 };
    let json = marshal_indent(&p, "  ").unwrap();
    assert!(json.contains("\n"));
    assert!(json.contains("\"x\": 1"));
}
