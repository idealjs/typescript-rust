use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_type_alias() {
    let content = r#"type T = number | string;"#;
    let mut s = Session::new_for_test("navigationBarItemsTypeAlias", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
