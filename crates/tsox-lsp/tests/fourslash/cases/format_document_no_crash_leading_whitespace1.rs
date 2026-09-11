use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_no_crash_leading_whitespace1() {
    let content = r#" \n"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashLeadingWhitespace1", content);
    fourslash::format_document(&mut s, "");
}
