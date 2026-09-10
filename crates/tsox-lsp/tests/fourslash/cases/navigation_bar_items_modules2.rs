use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_modules2() {
    let content = r#"namespace Test.A { }

namespace Test.B {
    class Foo { }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsModules2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
