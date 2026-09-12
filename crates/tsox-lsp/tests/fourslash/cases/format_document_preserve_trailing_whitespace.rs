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
    fourslash::configure_format_settings(&mut s, &[("trim_trailing_whitespace", "false")]);
    fourslash::format_document(&mut s, "");
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
