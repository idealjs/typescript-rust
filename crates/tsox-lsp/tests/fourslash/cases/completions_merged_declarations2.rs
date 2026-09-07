use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
