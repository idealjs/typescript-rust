use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_readonly1() {
    let content = r#"interface I {
  [|readonly|] prop: string;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReadonly1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
