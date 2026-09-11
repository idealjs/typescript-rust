use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_implicit_module() {
    let content = r#"       export class A {

       }"#;
    let mut s = Session::new_for_test("formatImplicitModule", content);
    // TODO: f.FormatDocument(t, "")
    // TODO: f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"export class A {"#);
    // TODO: f.GoToEOF(t)
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
