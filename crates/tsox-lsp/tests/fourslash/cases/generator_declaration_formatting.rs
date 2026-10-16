use tsox_lsp::fourslash::{self, Session};


#[test]
fn generator_declaration_formatting() {
    let content = r#"function    *g() { }/*1*/
var v = function    *() { };/*2*/"#;
    let mut s = Session::new_for_test("generatorDeclarationFormatting", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function* g() { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"var v = function*() { };"#);
}
