use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_async_computed_method() {
    let content = r#"class C {
    /*method*/async [0]() { }
}"#;
    let mut s = Session::new_for_test("formatAsyncComputedMethod", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "method");
    fourslash::verify_current_line_content(&mut s, r#"    async [0]() { }"#);
}
