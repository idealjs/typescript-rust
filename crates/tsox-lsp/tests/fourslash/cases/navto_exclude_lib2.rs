use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("navto_excludeLib2", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
