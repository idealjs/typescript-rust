use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.Configure(t, opts233)
    // TODO: f.FormatDocument(t, "")
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
