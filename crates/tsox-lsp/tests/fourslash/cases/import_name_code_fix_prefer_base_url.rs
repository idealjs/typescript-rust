use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_prefer_base_url() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "baseUrl": "./src" } }
// @Filename: /src/d0/d1/d2/file.ts
foo/**/;
// @Filename: /src/d0/a.ts
export const foo = 0;"#;
    let mut s = Session::new_for_test("importNameCodeFix_preferBaseUrl", content);
    fourslash::go_to_file(&mut s, "/src/d0/d1/d2/file.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
