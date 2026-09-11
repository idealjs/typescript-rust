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
    // TODO: f.FormatDocument(t, "")
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
    // TODO: opts400.FormatCodeSettings.IndentMultiLineObjectLiteralBeginningOnBlankLine = core.TSTrue
    // TODO: f.Configure(t, opts400)
    // TODO: f.FormatDocument(t, "")
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
