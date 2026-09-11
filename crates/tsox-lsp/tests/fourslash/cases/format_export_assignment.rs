use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_export_assignment() {
    let content = r#"export='foo';"#;
    let mut s = Session::new_for_test("formatExportAssignment", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"export = 'foo';"#);
}
