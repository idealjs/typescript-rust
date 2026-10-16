use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_remove_new_line_after_open_brace() {
    let content = r#"function foo()
{
}
if (true)
{
}"#;
    let mut s = Session::new_for_test("formatRemoveNewLineAfterOpenBrace", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"function foo() {
}
if (true) {
}"#);
}
