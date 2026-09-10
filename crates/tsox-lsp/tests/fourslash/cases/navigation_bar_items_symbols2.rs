use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_symbols2() {
    let content = r#"interface I {
    [Symbol.isRegExp]: string;
    [Symbol.iterator](): string;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsSymbols2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
