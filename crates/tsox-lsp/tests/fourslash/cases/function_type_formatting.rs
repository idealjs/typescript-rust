use tsox_lsp::fourslash::{self, Session};


#[test]
fn function_type_formatting() {
    let content = r#"var x: () =>           string/**/"#;
    let mut s = Session::new_for_test("functionTypeFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"var x: () => string;"#);
}
