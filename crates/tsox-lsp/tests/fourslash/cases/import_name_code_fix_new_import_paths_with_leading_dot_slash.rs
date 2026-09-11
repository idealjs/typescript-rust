use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_paths_with_leading_dot_slash() {
    let content = r#"// @Filename: /a.ts
[|foo|]
// @Filename: /thisHasPathMapping.ts
export function foo() {};
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "foo": ["././thisHasPathMapping"]
        }
    }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportPaths_withLeadingDotSlash", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
