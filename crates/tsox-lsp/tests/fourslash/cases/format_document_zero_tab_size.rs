use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_zero_tab_size() {
    let content = r#"function foo() {
    if (true) {
        var x = 1;
    }
}"#;
    let mut s = Session::new_for_test("formatDocumentZeroTabSize", content);
    // TODO: opts := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("tab_size", "0"), ("indent_size", "0"), ("convert_tabs_to_spaces", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, "function foo() {\nif (true) {\nvar x = 1;\n}\n}");
}
