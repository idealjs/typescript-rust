use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_remove_new_line_after_open_brace() {
    let content = r#"function foo()
{
}
if (true)
{
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"function foo() {
}
if (true) {
}"#,
    );
}
