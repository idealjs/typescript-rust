use tsox_lsp::fourslash::{self, Session};


#[test]
fn indent_after_function_closing_braces() {
    let content = r#"class foo {
    public f() {
        return 0;
    /*1*/}/*2*/
}"#;
    let mut s = Session::new_for_test("indentAfterFunctionClosingBraces", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert_line(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCurrentLineContent(t, `
}
