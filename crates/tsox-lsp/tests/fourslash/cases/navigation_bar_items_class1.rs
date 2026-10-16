use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_class1() {
    let content = r#"function Foo() {}
class Foo {}"#;
    let _s = Session::new_for_test("navigationBarItemsClass1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
