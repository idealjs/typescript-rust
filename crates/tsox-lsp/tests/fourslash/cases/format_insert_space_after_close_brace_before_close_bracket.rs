use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_insert_space_after_close_brace_before_close_bracket() {
    let content = r#"[{}]"#;
    let mut s = Session::new_for_test("formatInsertSpaceAfterCloseBraceBeforeCloseBracket", content);
    // TODO: opts122 := f.GetOptions()
    // TODO: opts122.FormatCodeSettings.InsertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets = core.TSTrue
    // TODO: f.Configure(t, opts122)
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"[ {} ]"#);
}
