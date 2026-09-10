use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn white_space_before_return_type_formatting() {
    let content = r#"var x: () =>     string/**/"#;
    let mut s = Session::new_for_test("whiteSpaceBeforeReturnTypeFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"var x: () => string;"#);
}
