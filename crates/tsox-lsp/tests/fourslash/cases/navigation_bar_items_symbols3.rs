use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_symbols3() {
    let content = r#"enum E {
    // No nav bar entry for this
    [Symbol.isRegExp] = 0
}"#;
    let mut s = Session::new_for_test("navigationBarItemsSymbols3", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
