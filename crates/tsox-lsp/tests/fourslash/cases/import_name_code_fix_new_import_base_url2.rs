use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_base_url2() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "./a"
    }
}
// @Filename: /a/b/x.ts
export function f1() { };
// @Filename: /a/c/y.ts
[|f1/*0*/();|]"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportBaseUrl2", content);
    fourslash::go_to_file(&mut s, "/a/c/y.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
