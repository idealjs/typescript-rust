use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_resolve_json_module() {
    let content = r#"// @resolveJsonModule: true
// @Filename: /a.ts
import text from "./message.json";
// @Filename: /message.json
{}"#;
    let _s = Session::new_for_test("getEditsForFileRename_resolveJsonModule", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/a.ts", "/src/a.ts", map[string]string{
}
