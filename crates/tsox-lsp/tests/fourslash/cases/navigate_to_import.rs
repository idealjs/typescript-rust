use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navigate_to_import() {
    let content = r#"// @lib: es5
// @Filename: library.ts
export function [|foo|]() {}
export function [|bar|]() {}
// @Filename: user.ts
import {foo, bar as [|baz|]} from './library';"#;
    let mut s = Session::new_for_test("navigateToImport", content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
