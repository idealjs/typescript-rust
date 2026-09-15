use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_symbols3() {
    let content = r#"enum E {
    // No nav bar entry for this
    [Symbol.isRegExp] = 0
}"#;
    let _s = Session::new_for_test("navigationBarItemsSymbols3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
