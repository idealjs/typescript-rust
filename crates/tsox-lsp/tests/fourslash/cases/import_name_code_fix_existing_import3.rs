use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_existing_import3() {
    let content = r#"[|import d, * as ns from "./module"   ;
f1/*0*/();|]
// @Filename: module.ts
export function f1() {}
export var v1 = 5;
export default var d1 = 6;"#;
    let _s = Session::new_for_test("importNameCodeFixExistingImport3", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
