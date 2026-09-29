use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_object_literal_open_curly_newline_assignment() {
    let content = r#"
var obj = {};
obj =
{
    prop: 3
};
 
var obj2 = obj ||
{
    prop: 0
}
"#;
    let mut s = Session::new_for_test("formattingObjectLiteralOpenCurlyNewlineAssignment", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
var obj = {};
obj =
{
    prop: 3
};

var obj2 = obj ||
{
    prop: 0
}
"#);
    // TODO: opts400 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("indent_multi_line_object_literal_beginning_on_blank_line", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
var obj = {};
obj =
    {
        prop: 3
    };

var obj2 = obj ||
    {
        prop: 0
    }
"#);
}
