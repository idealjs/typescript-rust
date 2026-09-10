use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_class5() {
    let content = r#"class Foo {}
let Foo = 1;"#;
    let mut s = Session::new_for_test("navigationBarItemsClass5", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
