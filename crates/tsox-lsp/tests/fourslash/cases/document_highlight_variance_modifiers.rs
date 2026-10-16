use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_variance_modifiers() {
    let content = r#"type TFoo<Value> = { value: Value };
type TBar<[|in|] [|out|] Value> = TFoo<Value>;"#;
    let _s = Session::new_for_test("documentHighlightVarianceModifiers", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
