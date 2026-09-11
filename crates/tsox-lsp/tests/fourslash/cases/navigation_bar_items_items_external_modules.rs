use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_items_external_modules() {
    let content = r#"export class Bar {
    public s: string;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsItemsExternalModules", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
