use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts131 := f.GetOptions()"]
#[test]
fn formatting_crash() {
    let content = r#"/**/module Default{ 
}"#;
    let mut s = Session::new_for_test("formattingCrash", content);
    // TODO: opts131 := f.GetOptions()
    // TODO: opts131.FormatCodeSettings.PlaceOpenBraceOnNewLineForFunctions = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts131)
    // TODO: opts199 := f.GetOptions()
    // TODO: opts199.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts199)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"module Default"#);
}
