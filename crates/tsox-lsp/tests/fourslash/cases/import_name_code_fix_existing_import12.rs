use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_existing_import12() {
    let content = r#"import [|{}|] from "./module";
f1/*0*/();
// @Filename: module.ts
export function f1() {}
export var v1 = 5;
export var v2 = 5;
export var v3 = 5;"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport12", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
