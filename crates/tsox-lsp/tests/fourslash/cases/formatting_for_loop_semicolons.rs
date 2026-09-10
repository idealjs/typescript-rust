use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts444 := f.GetOptions()"]
#[test]
fn formatting_for_loop_semicolons() {
    let content = r#"/*1*/for (;;) { }
/*2*/for (var x;x<0;x++) { }
/*3*/for (var x ;x<0 ;x++) { }"#;
    let mut s = Session::new_for_test("formattingForLoopSemicolons", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"for (; ;) { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"for (var x; x < 0; x++) { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"for (var x; x < 0; x++) { }"#);
    // TODO: opts444 := f.GetOptions()
    // TODO: opts444.FormatCodeSettings.InsertSpaceAfterSemicolonInForStatements = core.TSFalse
    fourslash::unsupported("Configure"); // f.Configure(t, opts444)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"for (;;) { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"for (var x;x < 0;x++) { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"for (var x;x < 0;x++) { }"#);
}
