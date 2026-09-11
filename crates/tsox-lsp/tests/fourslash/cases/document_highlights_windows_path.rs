use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights_windows_path() {
    let content = r#"//@Filename: C:\a\b\c.ts
var /*1*/[|x|] = 1;"#;
    let mut s = Session::new_for_test("documentHighlights_windowsPath", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{f.Ranges()[0].FileNam
}
