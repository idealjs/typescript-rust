use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_readonly2() {
    let content = r#"type T = {
  [|readonly|] prop: string;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReadonly2", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
