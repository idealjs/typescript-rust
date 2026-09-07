use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_try_finally() {
    let content = r#"if (true) try  {
    // ...
}   finally    {
    // ...
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"if (true) try {
    // ...
} finally {
    // ...
}"#,
    );
}
