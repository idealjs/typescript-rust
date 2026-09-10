use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn semicolon_formatting_nested_statements() {
    let content = r#"if (true)
if (true)/*parentOutsideBlock*/
if (true) {
if (true)/*directParent*/
var x = 0/*innermost*/
}"#;
    let mut s = Session::new_for_test("semicolonFormattingNestedStatements", content);
    fourslash::go_to_marker(&mut s, "innermost");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"        var x = 0;"#);
    fourslash::go_to_marker(&mut s, "directParent");
    fourslash::verify_current_line_content(&mut s, r#"    if (true)"#);
    fourslash::go_to_marker(&mut s, "parentOutsideBlock");
    fourslash::verify_current_line_content(&mut s, r#"if (true)"#);
}
