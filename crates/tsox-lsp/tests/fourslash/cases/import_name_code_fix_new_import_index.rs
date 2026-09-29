use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_index() {
    let content = r#"// @Filename: /a/index.ts
export const foo = 0;
// @Filename: /b.ts
[|/**/foo;|]"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportIndex", content);
    fourslash::go_to_file(&mut s, "/a/index.ts");
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
