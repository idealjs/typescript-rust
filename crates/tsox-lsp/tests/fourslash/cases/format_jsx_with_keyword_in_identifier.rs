use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_jsx_with_keyword_in_identifier() {
    let content = r#"// @Filename: /a.tsx
<div module-layout=""></div>"#;
    let mut s = Session::new_for_test("formatJsxWithKeywordInIdentifier", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"<div module-layout=""></div>"#);
}
