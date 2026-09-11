use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_missing_name2() {
    let content = r#"/**
 * This is a class.
 */
class /* But it has no name! */ {
    foo() {}
}"#;
    let mut s = Session::new_for_test("navigationBarItemsMissingName2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
