use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts123 := f.GetOptions()"]
#[test]
fn format_if_with_empty_condition() {
    let content = r#"if () {
}"#;
    let mut s = Session::new(content);
    // TODO: opts123 := f.GetOptions()
    // TODO: opts123.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts123)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"if ()
{
}"#,
    );
}
