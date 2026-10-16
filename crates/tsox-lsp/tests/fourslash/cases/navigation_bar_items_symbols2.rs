use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_symbols2() {
    let content = r#"interface I {
    [Symbol.isRegExp]: string;
    [Symbol.iterator](): string;
}"#;
    let _s = Session::new_for_test("navigationBarItemsSymbols2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
