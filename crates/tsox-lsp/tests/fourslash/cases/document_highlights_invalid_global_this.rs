use tsox_lsp::fourslash::Session;


#[test]
fn document_highlights_invalid_global_this() {
    let content = r#"declare global {
    export { globalThis as [|global|] }
}"#;
    let _s = Session::new_for_test("documentHighlightsInvalidGlobalThis", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
