use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_insert_space_after_close_brace_before_close_bracket() {
    let content = r#"[{}]"#;
    let mut s = Session::new_for_test("formatInsertSpaceAfterCloseBraceBeforeCloseBracket", content);
    // TODO: opts122 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_opening_and_before_closing_nonempty_brackets", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"[ {} ]"#);
}
