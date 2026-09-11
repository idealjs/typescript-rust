use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_string_literals() {
    let content = r#"var x = "[|string|]";
function f(a = "[|initial value|]") { }"#;
    let mut s = Session::new_for_test("getOccurrencesStringLiterals", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
