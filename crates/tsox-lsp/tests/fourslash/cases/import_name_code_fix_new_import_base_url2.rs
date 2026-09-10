use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_base_url2() {
    // TODO: t.Skip("Known failing fourslash test")
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
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
