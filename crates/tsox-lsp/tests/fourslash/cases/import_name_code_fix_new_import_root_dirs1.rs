use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_root_dirs1() {
    let content = r#"// @Filename: a/f1.ts
[|foo/*0*/();|]
// @Filename: a/b/index.ts
export function foo() {};
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "rootDirs": [
            "a"
        ]
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
