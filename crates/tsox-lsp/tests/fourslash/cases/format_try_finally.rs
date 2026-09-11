use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_try_finally() {
    let content = r#"if (true) try  {
    // ...
}   finally    {
    // ...
}"#;
    let mut s = Session::new_for_test("formatTryFinally", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"if (true) try {
    // ...
} finally {
    // ...
}"#);
}
