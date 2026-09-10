use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlights_invalid_global_this() {
    let content = r#"declare global {
    export { globalThis as [|global|] }
}"#;
    let mut s = Session::new_for_test("documentHighlightsInvalidGlobalThis", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
