use tsox_lsp::fourslash::{self, Session};


#[test]
fn declare_function() {
    let content = r#"// @filename: index.ts
declare function"#;
    let mut s = Session::new_for_test("declareFunction", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
