use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts148 := f.GetOptions()"]
#[test]
fn format_on_enter_open_brace_add_new_line() {
    let content = r#"if(true) {/*0*/}
if(false)/*1*/{
}"#;
    let mut s = Session::new_for_test("formatOnEnterOpenBraceAddNewLine", content);
    // TODO: opts148 := f.GetOptions()
    // TODO: opts148.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts148)
    fourslash::go_to_marker(&mut s, "0");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_file_content(&mut s, r#"if (true)
{
}
if(false){
}"#);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_file_content(&mut s, r#"if (true)
{
}
if (false)
{
}"#);
}
