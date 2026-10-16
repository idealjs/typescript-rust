use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_merged_declarations2() {
    let content = r#"class point {
    constructor(public x: number, public y: number) { }
}
namespace point {
    export var origin = new point(0, 0);
    export function equals(p1: point, p2: point) {
        return p1.x == p2.x && p1.y == p2.y;
    }
}
var p1 = new point(0, 0);
var p2 = point./*1*/origin;
var b = point./*2*/equals(p1, p2);"#;
    let mut s = Session::new_for_test("completionsMergedDeclarations2", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["origin"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["equals"], &[]);
}
