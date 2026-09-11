use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_existing_import5() {
    let content = r#"[|import "./module";
f1/*0*/();|]
// @Filename: module.ts
export function f1() {}
export var v1 = 5;"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport5", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
