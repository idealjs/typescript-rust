use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_document_no_crash_jsx_attr_unterminated_string() {
    let content = r#"// @Filename: /a.tsx
const x = <HangupButton customClass = 'ha
"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashJsxAttrUnterminatedString", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, "const x = <HangupButton customClass= 'ha\n");
}
