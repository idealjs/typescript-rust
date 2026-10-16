use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_in_try_catch_finally() {
    let content = r#"try 
{
    var x = 1/*1*/
}
catch (e) 
{
}"#;
    let mut s = Session::new_for_test("formatInTryCatchFinally", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"    var x = 1;"#);
}
