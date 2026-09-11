use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_functions_broken2() {
    let content = r#"function;
function f() {
    function;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctionsBroken2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
