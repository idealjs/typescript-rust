use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_after_multi_line_string() {
    let content = r#"class foo {
    stop() {
        var s = "hello\/*1*/
"/*2*/
    }
}"#;
    let mut s = Session::new_for_test("formattingAfterMultiLineString", content);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, "        var s = \"hello\\");
}
