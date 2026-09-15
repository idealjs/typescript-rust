use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_default_in_keyword() {
    let content = r#"[|case|]
[|default|]"#;
    let _s = Session::new_for_test("documentHighlightDefaultInKeyword", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
