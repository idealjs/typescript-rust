use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_tsconfig_include_add() {
    let content = r#"// @Filename: /src/tsconfig.json
{
    "include": ["dir"],
}
// @Filename: /src/dir/a.ts
"#;
    let mut s = Session::new_for_test("getEditsForFileRename_tsconfig_include_add", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/dir/a.ts", "/src/newDir/b.ts", map[string]string{
}
