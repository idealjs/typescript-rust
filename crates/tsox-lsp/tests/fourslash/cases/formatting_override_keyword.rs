use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_override_keyword() {
    let content = r#"class MyClass {
  override     myMethod() { };/*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    override myMethod() { };"#);
}
