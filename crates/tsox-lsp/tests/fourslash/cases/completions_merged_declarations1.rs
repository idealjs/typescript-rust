use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_merged_declarations1() {
    let content = r#"// @lib: es5
interface Point {
    x: number;
    y: number;
}
function point(x: number, y: number): Point {
    return { x: x, y: y };
}
namespace point {
    export var origin = point(0, 0);
    export function equals(p1: Point, p2: Point) {
        return p1.x == p2.x && p1.y == p2.y;
    }
}
var p1 = /*1*/point(0, 0);
var p2 = point./*2*/origin;
var b = point./*3*/equals(p1, p2);"#;
    let mut s = Session::new_for_test("completionsMergedDeclarations1", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["point"], &[]);
    // TODO: f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
