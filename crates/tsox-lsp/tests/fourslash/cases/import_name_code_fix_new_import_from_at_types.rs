use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_from_at_types() {
    let content = r#"[|f1/*0*/();|]
// @Filename: node_modules/@types/myLib/index.d.ts
export function f1() {}
export var v1 = 5;"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFromAtTypes", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
