use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/x/y.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
