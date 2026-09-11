use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export() {
    let content = r#"// @Filename: /foo-bar.ts
export default 0;
// @Filename: /b.ts
[|foo/**/Bar|]"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
