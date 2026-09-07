use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_implicit_module() {
    let content = r#"       export class A {

       }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"export class A {"#);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
