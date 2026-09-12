use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_close_brace() {
    let content = r#"class foo    {
    /**/"#;
    let mut s = Session::new_for_test("formattingOnCloseBrace", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "}");
    fourslash::go_to_bof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"class foo {"#);
    // TODO: }
}
