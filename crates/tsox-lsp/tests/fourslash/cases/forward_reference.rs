use tsox_lsp::fourslash::{self, Session};


#[test]
fn forward_reference() {
    let content = r#"function f() {
    var x = new t();
    x./**/
}
class t {
    public n: number;
}"#;
    let mut s = Session::new_for_test("forwardReference", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["n"]);
}
