use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_from_path_mapping() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /x/y.ts
foo;
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "@root/*": ["*"],
        }
    }
}"#;
    let mut s = Session::new_for_test("importNameCodeFix_fromPathMapping", content);
    fourslash::go_to_file(&mut s, "/x/y.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
