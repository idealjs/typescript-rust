use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlights_40082() {
    let content = r#"// @checkJs: true
export = (state, messages) => {
   export [|default|] {
   }
}"#;
    let mut s = Session::new_for_test("documentHighlights_40082", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
