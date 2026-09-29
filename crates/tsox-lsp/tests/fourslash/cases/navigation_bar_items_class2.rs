use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_class2() {
    let content = r#"class Foo {}
function Foo() {}"#;
    let _s = Session::new_for_test("navigationBarItemsClass2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
