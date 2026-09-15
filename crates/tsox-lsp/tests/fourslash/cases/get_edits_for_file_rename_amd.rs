use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_amd() {
    let content = r#"// @moduleResolution: classic
// @Filename: /src/user.ts
import { x } from "old";
// @Filename: /src/old.ts
export const x = 0;"#;
    let _s = Session::new_for_test("getEditsForFileRename_amd", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/new.ts", map[string]string{
}
