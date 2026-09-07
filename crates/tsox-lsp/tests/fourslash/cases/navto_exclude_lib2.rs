use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navto_exclude_lib2() {
    let content = r#"// @filename: /index.ts
import { someName as [|weirdName|] } from "bar";
// @filename: /tsconfig.json
{}
// @filename: /node_modules/bar/index.d.ts
export const someName: number;
// @filename: /node_modules/bar/package.json
{}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
