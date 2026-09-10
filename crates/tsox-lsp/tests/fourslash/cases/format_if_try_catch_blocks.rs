use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts187 := f.GetOptions()"]
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
    fourslash::unsupported("Configure"); // f.Configure(t, opts187)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
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
