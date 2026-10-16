use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_ambient_module() {
    let content = r#"// @Filename: /tsconfig.json
{}
// @Filename: /sub/types.d.ts
// @Symlink: /node_modules/sub/types.d.ts
declare module "sub" {
    declare export const abc: number
}
// @Filename: /sub/package.json
// @Symlink: /node_modules/sub/package.json
{ "types": "types.d.ts" }
// @Filename: /a.ts
import { abc } from "sub";"#;
    let _s = Session::new_for_test("getEditsForFileRename_ambientModule", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/a.ts", "/b.ts", map[string]string{}, nil /*preferences*/)
}
