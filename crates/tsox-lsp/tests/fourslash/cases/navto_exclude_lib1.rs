use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navto_exclude_lib1() {
    let content = r#"// @filename: /index.ts
import { weirdName as otherName } from "bar";
const [|weirdName|]: number = 1;
// @filename: /tsconfig.json
{}
// @filename: /node_modules/bar/index.d.ts
export const [|weirdName|];
// @filename: /node_modules/bar/package.json
{}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
