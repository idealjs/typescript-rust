use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts122 := f.GetOptions()"]
#[test]
fn format_insert_space_after_close_brace_before_close_bracket() {
    let content = r#"[{}]"#;
    let mut s = Session::new(content);
    // TODO: opts122 := f.GetOptions()
    // TODO: opts122.FormatCodeSettings.InsertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts122)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"[ {} ]"#);
}
