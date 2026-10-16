use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_async_class_method2() {
    let content = r#"class Foo {
    private    async     foo() {}
}"#;
    let mut s = Session::new_for_test("formatAsyncClassMethod2", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"class Foo {
    private async foo() { }
}"#);
}
