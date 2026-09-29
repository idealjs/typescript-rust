use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_implicit_module() {
    let content = r#"       export class A {

       }"#;
    let mut s = Session::new_for_test("formatImplicitModule", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_bof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"export class A {"#);
    fourslash::go_to_eof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
