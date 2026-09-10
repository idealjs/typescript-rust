use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_type_alias() {
    let content = r#"type T = number | string;"#;
    let mut s = Session::new_for_test("navigationBarItemsTypeAlias", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
