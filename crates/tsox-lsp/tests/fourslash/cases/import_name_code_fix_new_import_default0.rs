use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_default0() {
    let content = r#"[|f1/*0*/();|]
// @Filename: module.ts
export default function f1() { };"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportDefault0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
