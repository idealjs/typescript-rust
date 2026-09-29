use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_if_with_empty_condition() {
    let content = r#"if () {
}"#;
    let mut s = Session::new_for_test("formatIfWithEmptyCondition", content);
    // TODO: opts123 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"if ()
{
}"#);
}
