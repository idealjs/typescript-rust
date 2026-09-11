use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_base_url0() {
    let content = r#"[|f1/*0*/();|]
// @Filename: tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "./a"
    }
}
// @Filename: a/b.ts
export function f1() { };"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportBaseUrl0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
