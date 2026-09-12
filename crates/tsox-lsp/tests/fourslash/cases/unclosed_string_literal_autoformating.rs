use tsox_lsp::fourslash::{self, Session};


#[test]
fn unclosed_string_literal_autoformating() {
    let content = r#"var x = /*1*/"asd/*2*/
class Foo {
    /**/"#;
    let mut s = Session::new_for_test("unclosedStringLiteralAutoformating", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "}");
    // TODO: f.VerifyCurrentLineContent(t, `
}
