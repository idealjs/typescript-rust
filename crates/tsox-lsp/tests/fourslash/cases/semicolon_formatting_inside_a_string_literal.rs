use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn semicolon_formatting_inside_a_string_literal() {
    let content = r#"    var x = "string/**/"#;
    let mut s = Session::new_for_test("semicolonFormattingInsideAStringLiteral", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"   var x = "string;"#);
}
