use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_type_roots0() {
    let content = r#"// @Filename: a/f1.ts
[|foo/*0*/();|]
// @Filename: types/random/index.ts
export function foo() {};
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "typeRoots": [
            "./types"
        ]
    }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportTypeRoots0", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
