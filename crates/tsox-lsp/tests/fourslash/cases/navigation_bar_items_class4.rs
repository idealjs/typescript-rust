use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_class4() {
    let content = r#"// @allowJs: true
// @filename: /foo.js
class Foo {}
function Foo() {}"#;
    let mut s = Session::new_for_test("navigationBarItemsClass4", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
