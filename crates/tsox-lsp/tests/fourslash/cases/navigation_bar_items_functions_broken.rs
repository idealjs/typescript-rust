use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_functions_broken() {
    let content = r#"function f() {
    function;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctionsBroken", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
