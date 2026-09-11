use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigate_to_symbol_iterator() {
    let content = r#"// @lib: es5
class C {
    [|[Symbol.iterator]|]() {}
}"#;
    let mut s = Session::new_for_test("navigateToSymbolIterator", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
