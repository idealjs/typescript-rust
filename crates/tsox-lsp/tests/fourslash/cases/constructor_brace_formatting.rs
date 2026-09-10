use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn constructor_brace_formatting() {
    let content = r#"class X {
    constructor () {}/*target*/
 /**/"#;
    let mut s = Session::new_for_test("constructorBraceFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Insert"); // f.Insert(t, "}")
}
