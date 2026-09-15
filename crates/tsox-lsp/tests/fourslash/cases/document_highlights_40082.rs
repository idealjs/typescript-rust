use tsox_lsp::fourslash::Session;


#[test]
fn document_highlights_40082() {
    let content = r#"// @checkJs: true
export = (state, messages) => {
   export [|default|] {
   }
}"#;
    let _s = Session::new_for_test("documentHighlights_40082", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
