use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_multiline_template_strings() {
    let content = r#"const foo = ` + "`" + "#;
    // TODO: a
    // TODO: [|b|]
    // TODO: c
    // TODO: ` + "`" + ``
    let mut s = Session::new_for_test("documentHighlightMultilineTemplateStrings", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
