use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_private_members2() {
    let content = r#"class Foo {
    private y;
    constructor(private x) {}
    method() { this./*1*/; }
}
var f:Foo;
f./*2*/"#;
    let mut s = Session::new_for_test("completionListPrivateMembers2", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["method", "x", "y"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["method"]);
}
