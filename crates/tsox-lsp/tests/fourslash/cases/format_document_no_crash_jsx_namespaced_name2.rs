use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_no_crash_jsx_namespaced_name2() {
    let content = r#"// @Filename: /a.tsx
const x = <A my-ns:attr="val" />;
"#;
    let mut s = Session::new_for_test("formatDocumentNoCrashJsxNamespacedName2", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, "const x = <A my-ns:attr=\"val\" />;\n");
}
