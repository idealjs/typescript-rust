use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_type_alias() {
    let content = r#"type T = number | string;"#;
    let _s = Session::new_for_test("navigationBarItemsTypeAlias", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
