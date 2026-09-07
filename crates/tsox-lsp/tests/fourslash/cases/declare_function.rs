use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn declare_function() {
    let content = r#"// @filename: index.ts
declare function"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
