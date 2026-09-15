use tsox_lsp::fourslash::Session;


#[test]
fn get_edits_for_file_rename_sub_dir() {
    let content = r#"// @Filename: /src/foo/a.ts

// @Filename: /src/old.ts
import a from "./foo/a";"#;
    let _s = Session::new_for_test("getEditsForFileRename_subDir", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/src/old.ts", "/src/dir/new.ts", map[string]string{
}
