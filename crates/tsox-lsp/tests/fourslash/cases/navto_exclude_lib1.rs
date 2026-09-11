use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("navto_excludeLib1", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
