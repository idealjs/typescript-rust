use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_type_roots1() {
    let content = r#"// @Filename: a/f1.ts
[|foo/*0*/();|]
// @Filename: types/random/index.ts
export function foo() {};
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "typeRoots": [
            "./types"
        ]
    }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportTypeRoots1", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
