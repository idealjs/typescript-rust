use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_paths_with_parent_relative_path() {
    let content = r#"// @Filename: /src/a.ts
[|foo|]
// @Filename: /thisHasPathMapping.ts
export function foo() {};
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "src",
        "paths": {
            "foo": ["..\\thisHasPathMapping"]
        }
    }
}"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportPaths_withParentRelativePath", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
