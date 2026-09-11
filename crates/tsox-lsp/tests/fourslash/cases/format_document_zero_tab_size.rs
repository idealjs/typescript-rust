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
    // TODO: opts.FormatCodeSettings.TabSize = 0
    // TODO: opts.FormatCodeSettings.IndentSize = 0
    // TODO: opts.FormatCodeSettings.ConvertTabsToSpaces = core.TSTrue
    // TODO: f.Configure(t, opts)
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, "function foo() {\nif (true) {\nvar x = 1;\n}\n}");
}
