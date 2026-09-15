use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_casing() {
    let content = r#"// @Filename: /a.ts
import { foo } from "./dir/fOo";
// @Filename: /dir/fOo.ts
export const foo = 0;"#;
    let _s = Session::new_for_test("getEditsForFileRename_casing", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/dir", "/newDir", map[string]string{
}
