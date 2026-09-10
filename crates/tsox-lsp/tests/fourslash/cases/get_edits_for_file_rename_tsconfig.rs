use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWillRenameFilesEdits"]
#[test]
fn get_edits_for_file_rename_tsconfig() {
    let content = r#"// @Filename: /src/tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "./old",
        "paths": {
            "foo": ["old"],
        },
        "rootDir": "old",
        "rootDirs": ["old"],
        "typeRoots": ["old"],
    },
    "files": ["old/a.ts"],
    "include": ["old/*.ts"],
    "exclude": ["old"],
}
// @Filename: /src/old/someFile.ts
"#;
    let mut s = Session::new_for_test("getEditsForFileRename_tsconfig", content);
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/old", "/src/new", map[string]string{
}
