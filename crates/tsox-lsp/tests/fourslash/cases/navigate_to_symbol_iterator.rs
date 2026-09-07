use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navigate_to_symbol_iterator() {
    let content = r#"// @lib: es5
class C {
    [|[Symbol.iterator]|]() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
