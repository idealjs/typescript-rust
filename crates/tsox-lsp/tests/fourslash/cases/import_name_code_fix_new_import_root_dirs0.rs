use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_root_dirs0() {
    let content = r#"// @Filename: a/f1.ts
[|foo/*0*/();|]
// @Filename: b/c/f2.ts
export function foo() {};
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "rootDirs": [
            "a",
            "b/c"
        ]
    }
}"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportRootDirs0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
