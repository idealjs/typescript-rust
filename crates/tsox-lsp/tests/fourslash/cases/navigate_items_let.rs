use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigate_items_let() {
    let content = r#"// @noLib: true
let [|c|] = 10;
function foo() {
    let [|d|] = 10;
}"#;
    let mut s = Session::new_for_test("navigateItemsLet", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
