use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_symbols1() {
    let content = r#"class C {
    [Symbol.isRegExp] = 0;
    [Symbol.iterator]() { }
    get [Symbol.isConcatSpreadable]() { }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsSymbols1", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
