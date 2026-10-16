use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_destructured_binding() {
    let content = r#"
function f({ /*1*/x }: { x: number }) {}
function g([/*2*/y]: number[]) {}
function h({ a: { /*3*/b } }: { a: { b: string } }) {}
const { /*4*/c } = { c: 42 };
let { /*5*/d } = { d: "hello" };
var { /*6*/e } = { e: true };
"#;
    let mut s = Session::new_for_test("quickInfoDestructuredBinding", content);
    // TODO: // Destructured object binding parameters should show "(parameter)" not "var"
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) x: number", "");
    // TODO: // Destructured array binding parameters should show "(parameter)" not "var"
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) y: number", "");
    // TODO: // Nested destructured parameters should also show "(parameter)"
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) b: string", "");
    // TODO: // Destructured const/let/var bindings should show their proper keyword
    fourslash::verify_quick_info_at(&mut s, "4", "const c: number", "");
    fourslash::verify_quick_info_at(&mut s, "5", "let d: string", "");
    fourslash::verify_quick_info_at(&mut s, "6", "var e: boolean", "");
}
