use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_unaffected_non_relative_path() {
    let content = r#"// @Filename: /sub/a.ts
export const a = 1;
// @Filename: /sub/b.ts
import { a } from "sub/a";
// @Filename: /tsconfig.json
{"compilerOptions":{"paths":{"*":["*"]}}}"#;
    let _s = Session::new_for_test("getEditsForFileRename_unaffectedNonRelativePath", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/sub/b.ts", "/sub/c/d.ts", map[string]string{}, nil /*preferences*/
}
