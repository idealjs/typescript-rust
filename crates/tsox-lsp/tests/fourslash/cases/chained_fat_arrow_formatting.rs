use tsox_lsp::fourslash::{self, Session};


#[test]
fn chained_fat_arrow_formatting() {
    let content = r#"var fn = () => () => null/**/"#;
    let mut s = Session::new_for_test("chainedFatArrowFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"var fn = () => () => null;"#);
}
