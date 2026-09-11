use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_existing_import4() {
    let content = r#"[|import d from "./module";
f1/*0*/();|]
// @Filename: module.ts
export function f1() {}
export var v1 = 5;
export default var d1 = 6;"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport4", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
