use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_preferences() {
    let content = r#"// @Filename: /dir/a.ts
export const a = 0;
// @Filename: /dir/b.ts
import {} from "dir/a";
import {} from 'dir/a';
// @Filename: /tsconfig.json
{"compilerOptions":{"paths":{"*":["*"]}}}"#;
    let mut s = Session::new_for_test("getEditsForFileRename_preferences", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/dir/a.ts", "/dir/a1.ts", map[string]string{
}
