use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_computed_property_name() {
    let content = r#"function F(key, value) {
    return {
        [key]: value,
        "prop": true
    }
}"#;
    let mut s = Session::new_for_test("navigationBarComputedPropertyName", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
