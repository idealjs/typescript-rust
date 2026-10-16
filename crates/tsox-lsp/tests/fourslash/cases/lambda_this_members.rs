use tsox_lsp::fourslash::{self, Session};


#[test]
fn lambda_this_members() {
    let content = r#"class Foo {
    a: number;
    b() {
        var x = () => {
            this./**/;
        }
    }
}"#;
    let mut s = Session::new_for_test("lambdaThisMembers", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a", "b"]);
}
