use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_if_with_empty_condition() {
    let content = r#"if () {
}"#;
    let mut s = Session::new_for_test("formatIfWithEmptyCondition", content);
    // TODO: opts123 := f.GetOptions()
    // TODO: opts123.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    // TODO: f.Configure(t, opts123)
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"if ()
{
}"#);
}
