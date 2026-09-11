use tsox_lsp::fourslash::{self, Session};


#[test]
fn navto_exclude_lib3() {
    let content = r#"// @filename: /index.ts
function [|parseInt|](s: string): number {}"#;
    let mut s = Session::new_for_test("navto_excludeLib3", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
