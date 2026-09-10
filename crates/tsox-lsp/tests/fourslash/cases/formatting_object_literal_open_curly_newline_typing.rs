use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn formatting_object_literal_open_curly_newline_typing() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"
var varName =/**/
"#;
    let mut s = Session::new_for_test("formattingObjectLiteralOpenCurlyNewlineTyping", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Insert"); // f.Insert(t, "\n{")
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"
var varName =
{
    a: 1
};
"#);
    // TODO: }
}
