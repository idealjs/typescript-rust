use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_case_insensitive() {
    let content = r#"// @useCaseSensitiveFileNames: false
// @Filename: /a.ts
export const a = 0;
// @Filename: /b.ts
import { a } from "./A";"#;
    let _s = Session::new_for_test("getEditsForFileRename_caseInsensitive", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/a.ts", "/eh.ts", map[string]string{
}
