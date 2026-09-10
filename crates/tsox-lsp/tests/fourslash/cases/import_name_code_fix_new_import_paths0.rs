use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_paths0() {
    let content = r#"[|foo/*0*/();|]
// @Filename: folder_a/f2.ts
export function foo() {};
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "a": [ "folder_a/f2" ]
        }
    }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportPaths0", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
