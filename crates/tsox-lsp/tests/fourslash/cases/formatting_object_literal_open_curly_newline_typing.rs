use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn formatting_object_literal_open_curly_newline_typing() {
    let content = r#"
var varName =/**/
"#;
    let mut s = Session::new_for_test("formattingObjectLiteralOpenCurlyNewlineTyping", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Insert(t, "\n{")
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
var varName =
{
    a: 1
};
"#);
    // TODO: }
}
