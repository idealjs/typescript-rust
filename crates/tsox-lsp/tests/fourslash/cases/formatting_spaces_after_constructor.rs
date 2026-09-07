use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts319 := f.GetOptions()"]
#[test]
fn formatting_spaces_after_constructor() {
    let content = r#"/*1*/class test { constructor                   () { } }
/*2*/class test { constructor                   () { } }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"class test { constructor() { } }"#);
    // TODO: opts319 := f.GetOptions()
    // TODO: opts319.FormatCodeSettings.InsertSpaceAfterConstructor = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts319)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"class test { constructor () { } }"#);
}
