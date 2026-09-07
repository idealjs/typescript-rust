use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_tsconfig_empty_include() {
    let content = r#"// @Filename: /a/foo.ts
const x = 1
// @Filename: /a/tsconfig.json
{ "include": [] }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/a/foo.ts", "/a/bar.ts", map[string]string{}, nil /*preferences*/)
}
