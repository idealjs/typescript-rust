use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_on_semi_colon_after_break() {
    let content = r#"for (var a in b) {
break/**/
}"#;
    let mut s = Session::new_for_test("formatOnSemiColonAfterBreak", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"    break;"#);
}
