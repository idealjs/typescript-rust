use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_object_literal_open_curly_newline() {
    let content = r#"
var clear =
{
    outerKey:
    {
        innerKey: 1,
        innerKey2:
            2
    }
};
"#;
    let mut s = Session::new_for_test("formattingObjectLiteralOpenCurlyNewline", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
var clear =
{
    outerKey:
    {
        innerKey: 1,
        innerKey2:
            2
    }
};
"#);
    // TODO: opts444 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("indent_multi_line_object_literal_beginning_on_blank_line", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
var clear =
    {
        outerKey:
            {
                innerKey: 1,
                innerKey2:
                    2
            }
    };
"#);
}
