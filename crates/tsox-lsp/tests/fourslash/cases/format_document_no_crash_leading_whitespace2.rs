use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_no_crash_leading_whitespace2() {
    let content = r#" 
;"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashLeadingWhitespace2", content);
    fourslash::format_document(&mut s, "");
}
