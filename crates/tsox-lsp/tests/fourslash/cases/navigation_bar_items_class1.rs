use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_class1() {
    let content = r#"function Foo() {}
class Foo {}"#;
    let mut s = Session::new_for_test("navigationBarItemsClass1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
