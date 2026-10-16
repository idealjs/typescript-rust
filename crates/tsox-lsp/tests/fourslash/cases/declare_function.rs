use tsox_lsp::fourslash::Session;


#[test]
fn declare_function() {
    let content = r#"// @filename: index.ts
declare function"#;
    let _s = Session::new_for_test("declareFunction", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
