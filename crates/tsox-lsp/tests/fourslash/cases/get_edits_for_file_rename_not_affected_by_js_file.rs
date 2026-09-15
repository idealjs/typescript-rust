use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_not_affected_by_js_file() {
    let content = r#"// @Filename: /a.ts
export const x = 0;
// @Filename: /a.js
exports.x = 0;
// @Filename: /b.ts
import { x } from "./a";"#;
    let _s = Session::new_for_test("getEditsForFileRename_notAffectedByJsFile", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/a.ts", "/a2.ts", map[string]string{
}
