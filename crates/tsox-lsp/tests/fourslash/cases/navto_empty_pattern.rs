use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navto_empty_pattern() {
    let content = r#"// @filename: foo.ts
const [|x|]: number = 1;
function [|y|](x: string): string { return x; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
