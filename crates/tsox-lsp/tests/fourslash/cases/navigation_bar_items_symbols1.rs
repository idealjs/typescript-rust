use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_symbols1() {
    let content = r#"class C {
    [Symbol.isRegExp] = 0;
    [Symbol.iterator]() { }
    get [Symbol.isConcatSpreadable]() { }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsSymbols1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
