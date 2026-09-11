use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_empty_constructors() {
    let content = r#"class Test {
    constructor() {
    }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsEmptyConstructors", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
