use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts := f.GetOptions()"]
#[test]
fn format_document_zero_tab_size() {
    let content = r#"function foo() {
    if (true) {
        var x = 1;
    }
}"#;
    let mut s = Session::new(content);
    // TODO: opts := f.GetOptions()
    // TODO: opts.FormatCodeSettings.TabSize = 0
    // TODO: opts.FormatCodeSettings.IndentSize = 0
    // TODO: opts.FormatCodeSettings.ConvertTabsToSpaces = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        "function foo() {\nif (true) {\nvar x = 1;\n}\n}",
    );
}
