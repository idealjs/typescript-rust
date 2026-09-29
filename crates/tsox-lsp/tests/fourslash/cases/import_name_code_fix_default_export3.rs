use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export3() {
    let content = r#"// @Filename: /foo-bar/index.ts
export default 0;
// @Filename: /b.ts
[|foo/**/Bar|]"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport3", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
