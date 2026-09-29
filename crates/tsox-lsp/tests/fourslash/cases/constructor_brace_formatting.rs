use tsox_lsp::fourslash::{self, Session};


#[test]
fn constructor_brace_formatting() {
    let content = r#"class X {
    constructor () {}/*target*/
 /**/"#;
    let mut s = Session::new_for_test("constructorBraceFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "}");
    fourslash::go_to_marker(&mut s, "target");
    fourslash::verify_current_line_content(&mut s, r#"    constructor() { }"#);
}
