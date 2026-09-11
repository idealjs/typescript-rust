use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_async_class_method1() {
    let content = r#"class Foo {
    async     foo() {}
}"#;
    let mut s = Session::new_for_test("formatAsyncClassMethod1", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"class Foo {
    async foo() { }
}"#);
}
