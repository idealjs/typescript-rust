use tsox_lsp::fourslash::Session;


#[test]
fn export_in_labeled_statement() {
    let content = r#"// @Filename: a.ts
subTitle:
[|export|] const title: string"#;
    let _s = Session::new_for_test("exportInLabeledStatement", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{f.Ranges()[0].FileNam
}
