use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn single_line_type_literal_formatting() {
    let content = r#"function of1(b: { r: { c: number/**/"#;
    let mut s = Session::new_for_test("singleLineTypeLiteralFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"function of1(b: { r: { c: number;"#);
    // TODO: }
}
