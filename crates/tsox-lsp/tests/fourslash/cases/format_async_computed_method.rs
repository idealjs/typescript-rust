use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_async_computed_method() {
    let content = r#"class C {
    /*method*/async [0]() { }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "method");
    fourslash::verify_current_line_content(&mut s, r#"    async [0]() { }"#);
}
