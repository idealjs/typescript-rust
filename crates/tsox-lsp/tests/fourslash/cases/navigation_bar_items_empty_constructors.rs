use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_empty_constructors() {
    let content = r#"class Test {
    constructor() {
    }
}"#;
    let _s = Session::new_for_test("navigationBarItemsEmptyConstructors", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
