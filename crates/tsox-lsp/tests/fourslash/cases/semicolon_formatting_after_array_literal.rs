use tsox_lsp::fourslash::{self, Session};


#[test]
fn semicolon_formatting_after_array_literal() {
    let content = r#"[1,2]/**/"#;
    let mut s = Session::new_for_test("semicolonFormattingAfterArrayLiteral", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"[1, 2];"#);
}
