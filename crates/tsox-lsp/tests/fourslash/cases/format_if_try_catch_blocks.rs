use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_if_try_catch_blocks() {
    let content = r#"try {
}
catch {
}

try {
}
catch (e) {
}"#;
    let mut s = Session::new_for_test("formatIfTryCatchBlocks", content);
    // TODO: opts187 := f.GetOptions()
    // TODO: opts187.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    // TODO: f.Configure(t, opts187)
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"try
{
}
catch
{
}

try
{
}
catch (e)
{
}"#);
}
