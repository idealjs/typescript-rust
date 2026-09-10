use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_functions_broken2() {
    let content = r#"function;
function f() {
    function;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctionsBroken2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
