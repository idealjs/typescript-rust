use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_any_type_literal() {
    let content = r#"function foo(x: { } /*objLit*/){
/**/"#;
    let mut s = Session::new_for_test("formatAnyTypeLiteral", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "}");
    fourslash::go_to_marker(&mut s, "objLit");
    fourslash::verify_current_line_content(&mut s, r#"function foo(x: {}) {"#);
    // TODO: }
}
