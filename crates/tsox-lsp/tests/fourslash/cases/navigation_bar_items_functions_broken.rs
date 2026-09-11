use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_functions_broken() {
    let content = r#"function f() {
    function;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctionsBroken", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
