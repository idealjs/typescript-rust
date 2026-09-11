use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_string_literal_types() {
    let content = r#"function foo(a: "[|option 1|]") { }
foo("[|option 1|]");"#;
    let mut s = Session::new_for_test("getOccurrencesStringLiteralTypes", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
