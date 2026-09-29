use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("importNameCodeFixNewImportPaths0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
