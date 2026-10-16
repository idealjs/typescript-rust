use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_no_crash_jsx_namespaced_name1() {
    let content = r#"// @Filename: /a.tsx
const x = <foo:bar />;
"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashJsxNamespacedName1", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, "const x = <foo:bar />;\n");
}
