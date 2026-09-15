use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_functions_broken() {
    let content = r#"function f() {
    function;
}"#;
    let _s = Session::new_for_test("navigationBarItemsFunctionsBroken", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
