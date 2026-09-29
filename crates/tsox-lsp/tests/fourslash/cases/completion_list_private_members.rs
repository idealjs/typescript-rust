use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_private_members() {
    let content = r#"class Foo {
    private x;
}

class Bar extends Foo {
    private y;
    foo() {
        this./**/
    }
}"#;
    let mut s = Session::new_for_test("completionListPrivateMembers", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo", "y"]);
}
