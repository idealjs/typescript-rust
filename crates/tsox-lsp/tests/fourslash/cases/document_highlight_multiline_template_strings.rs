use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_multiline_template_strings() {
    let content = r#"const foo = `
    a
    [|b|]
    c
`"#;
    let _s = Session::new_for_test("documentHighlightMultilineTemplateStrings", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
