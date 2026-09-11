use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_class3() {
    let content = r#"// @allowJs: true
// @filename: /foo.js
function Foo() {}
class Foo {}"#;
    let mut s = Session::new_for_test("navigationBarItemsClass3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
