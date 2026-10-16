use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_no_crash_short_last_line() {
    let content = r#"type X = {
	b}"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashShortLastLine", content);
    fourslash::format_document(&mut s, "");
}
