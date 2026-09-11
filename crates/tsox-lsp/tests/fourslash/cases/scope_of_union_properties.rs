use tsox_lsp::fourslash::{self, Session};


#[test]
fn scope_of_union_properties() {
    let content = r#"function f(s: string | number) {
    s.constr/*1*/uctor
}"#;
    let mut s = Session::new_for_test("scopeOfUnionProperties", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
