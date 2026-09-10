use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_items_external_modules() {
    let content = r#"export class Bar {
    public s: string;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsItemsExternalModules", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
