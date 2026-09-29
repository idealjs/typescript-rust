use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_class5() {
    let content = r#"class Foo {}
let Foo = 1;"#;
    let _s = Session::new_for_test("navigationBarItemsClass5", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
