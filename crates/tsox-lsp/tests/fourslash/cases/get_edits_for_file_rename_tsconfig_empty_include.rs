use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_edits_for_file_rename_tsconfig_empty_include() {
    let content = r#"// @Filename: /a/foo.ts
const x = 1
// @Filename: /a/tsconfig.json
{ "include": [] }"#;
    let mut s = Session::new_for_test("getEditsForFileRename_tsconfig_empty_include", content);
    // TODO: f.VerifyWillRenameFilesEdits(t, "/a/foo.ts", "/a/bar.ts", map[string]string{}, nil /*preferences*/)
}
