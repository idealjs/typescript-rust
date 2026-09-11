use tsox_lsp::fourslash::{self, Session};


#[test]
fn navto_empty_pattern() {
    let content = r#"// @filename: foo.ts
const [|x|]: number = 1;
function [|y|](x: string): string { return x; }"#;
    let mut s = Session::new_for_test("navto_emptyPattern", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
