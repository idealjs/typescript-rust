use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_resolve_json_module() {
    let content = r#"// @resolveJsonModule: true
// @Filename: /a.ts
import text from "./message.json";
// @Filename: /message.json
{}"#;
    let mut s = Session::new_for_test("getEditsForFileRename_resolveJsonModule", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/a.ts", "/src/a.ts", map[string]string{
}
