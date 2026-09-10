use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_protected_members() {
    let content = r#"class Base {
    protected y;
    constructor(protected x) {}
    method() { this./*1*/; }
}
class D1 extends Base {
    protected z;
    method1() { this./*2*/; }
}
class D2 extends Base {
    method2() { this./*3*/; }
}
class D3 extends D1 {
    method2() { this./*4*/; }
}
var b: Base;
f./*5*/"#;
    let mut s = Session::new_for_test("completionListProtectedMembers", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["y", "x", "method"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["z", "method1", "y", "x", "method"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["method2", "y", "x", "method"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["method2", "z", "method1", "y", "x", "method"]);
    fourslash::verify_completions_empty_at(&mut s, Some("5"));
}
