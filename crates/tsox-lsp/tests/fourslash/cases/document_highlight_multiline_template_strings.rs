use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: [|b|]"]
#[test]
fn document_highlight_multiline_template_strings() {
    let content = r#"const foo = ` + "`" + "#;
    // TODO: a
    // TODO: [|b|]
    // TODO: c
    // TODO: ` + "`" + ``
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
