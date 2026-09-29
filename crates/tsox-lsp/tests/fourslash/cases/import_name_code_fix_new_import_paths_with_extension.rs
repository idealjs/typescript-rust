use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_paths_with_extension() {
    let content = r#"// @Filename: /src/a.ts
[|foo|]
// @Filename: /src/thisHasPathMapping.ts
export function foo() {};
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "foo": ["src/thisHasPathMapping.ts"]
        }
    }
}"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportPaths_withExtension", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
