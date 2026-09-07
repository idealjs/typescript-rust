use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn generator_declaration_formatting() {
    let content = r#"function    *g() { }/*1*/
var v = function    *() { };/*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function* g() { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"var v = function*() { };"#);
}
