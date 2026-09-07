use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts211 := f.GetOptions()"]
#[test]
fn formatting_single_line_with_new_line_option_set() {
    let content = r#"/*1*/namespace Default{}
/*2*/function foo(){}
/*3*/if (true){}
/*4*/function boo() {
}"#;
    let mut s = Session::new(content);
    // TODO: opts211 := f.GetOptions()
    // TODO: opts211.FormatCodeSettings.PlaceOpenBraceOnNewLineForFunctions = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts211)
    // TODO: opts279 := f.GetOptions()
    // TODO: opts279.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts279)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"namespace Default { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"function foo() { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"if (true) { }"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"function boo()"#);
}
