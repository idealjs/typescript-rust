use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_override_keyword() {
    let content = r#"class MyClass {
  override     myMethod() { };/*1*/
}"#;
    let mut s = Session::new_for_test("formattingOverrideKeyword", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    override myMethod() { };"#);
}
