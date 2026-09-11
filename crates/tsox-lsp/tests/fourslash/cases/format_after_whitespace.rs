use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_after_whitespace() {
    let content = r#"function foo()
{
    var bar;
    /*1*/
}"#;
    let mut s = Session::new_for_test("formatAfterWhitespace", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.InsertLine(t, "")
    fourslash::verify_current_file_content(&mut s, r#"function foo()
{
    var bar;


}"#);
}
