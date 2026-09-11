use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_base_url1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "./a"
    }
}
// @Filename: /a/b/x.ts
export function f1() { };
// @Filename: /a/b/y.ts
[|f1/*0*/();|]"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportBaseUrl1", content);
    fourslash::go_to_file(&mut s, "/a/b/y.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
