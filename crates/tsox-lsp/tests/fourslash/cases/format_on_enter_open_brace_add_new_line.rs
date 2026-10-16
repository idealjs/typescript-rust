use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_on_enter_open_brace_add_new_line() {
    let content = r#"if(true) {/*0*/}
if(false)/*1*/{
}"#;
    let mut s = Session::new_for_test("formatOnEnterOpenBraceAddNewLine", content);
    // TODO: opts148 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    fourslash::go_to_marker(&mut s, "0");
    fourslash::insert_line(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"if (true)
{
}
if(false){
}"#);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert_line(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"if (true)
{
}
if (false)
{
}"#);
}
