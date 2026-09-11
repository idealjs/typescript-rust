use tsox_lsp::fourslash::{self, Session};


#[test]
fn export_in_labeled_statement() {
    let content = r#"// @Filename: a.ts
subTitle:
[|export|] const title: string"#;
    let mut s = Session::new_for_test("exportInLabeledStatement", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{f.Ranges()[0].FileNam
}
