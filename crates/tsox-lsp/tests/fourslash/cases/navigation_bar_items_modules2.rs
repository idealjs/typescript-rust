use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_modules2() {
    let content = r#"namespace Test.A { }

namespace Test.B {
    class Foo { }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsModules2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
