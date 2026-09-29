use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_nested_class_with_open_brace_on_new_lines() {
    let content = r#"module A
{
    class B {
        /*1*/
}"#;
    let mut s = Session::new_for_test("formatNestedClassWithOpenBraceOnNewLines", content);
    // TODO: opts168 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    // TODO: opts232 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_functions", "true")]);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "}");
    fourslash::verify_current_file_content(&mut s, r#"module A
{
    class B
    {
    }
}"#);
}
