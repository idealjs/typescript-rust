use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts233 := f.GetOptions()"]
#[test]
fn format_document_preserve_trailing_whitespace() {
    let content = r#"
var a;     
var b     
     
//     
function b(){     
    while(true){     
    }     
}     
"#;
    let mut s = Session::new_for_test("formatDocumentPreserveTrailingWhitespace", content);
    // TODO: opts233 := f.GetOptions()
    // TODO: opts233.FormatCodeSettings.TrimTrailingWhitespace = core.TSFalse
    fourslash::unsupported("Configure"); // f.Configure(t, opts233)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"
var a;     
var b     
     
//     
function b() {     
    while (true) {     
    }     
}     
"#);
}
