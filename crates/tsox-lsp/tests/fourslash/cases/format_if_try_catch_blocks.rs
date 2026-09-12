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
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
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
