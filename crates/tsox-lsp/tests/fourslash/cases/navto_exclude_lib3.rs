use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navto_exclude_lib3() {
    let content = r#"// @filename: /index.ts
function [|parseInt|](s: string): number {}"#;
    let mut s = Session::new_for_test("navto_excludeLib3", content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
