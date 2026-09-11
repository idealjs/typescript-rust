use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_get_canonical_file_name() {
    let content = r#"// @Filename: /howNow/node_modules/brownCow/index.d.ts
export const foo: number;
// @Filename: /howNow/a.ts
foo;"#;
    let mut s = Session::new_for_test("importNameCodeFix_getCanonicalFileName", content);
    fourslash::go_to_file(&mut s, "/howNow/a.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
