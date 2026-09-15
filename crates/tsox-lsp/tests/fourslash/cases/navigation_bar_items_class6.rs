use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_class6() {
    let content = r#"function Z() { }

Z.foo = 42

class Z { }"#;
    let _s = Session::new_for_test("navigationBarItemsClass6", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
