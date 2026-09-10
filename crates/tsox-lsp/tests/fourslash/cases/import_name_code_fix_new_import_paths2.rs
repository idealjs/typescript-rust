use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_paths2() {
    let content = r#"[|foo/*0*/();|]
// @Filename: folder_b/index.ts
export function foo() {};
// @Filename: tsconfig.path.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "b": [ "folder_b/index" ]
        }
    }
}
// @Filename: tsconfig.json
{
    "extends": "./tsconfig.path",
    "compilerOptions": { }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportPaths2", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
