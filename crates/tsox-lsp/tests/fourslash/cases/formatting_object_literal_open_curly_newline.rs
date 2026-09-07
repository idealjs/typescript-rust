use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts444 := f.GetOptions()"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
var clear =
{
    outerKey:
    {
        innerKey: 1,
        innerKey2:
            2
    }
};
"#,
    );
    // TODO: opts444 := f.GetOptions()
    // TODO: opts444.FormatCodeSettings.IndentMultiLineObjectLiteralBeginningOnBlankLine = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts444)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
var clear =
    {
        outerKey:
            {
                innerKey: 1,
                innerKey2:
                    2
            }
    };
"#,
    );
}
