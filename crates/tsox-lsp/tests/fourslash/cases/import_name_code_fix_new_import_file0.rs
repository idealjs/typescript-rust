use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_file0() {
    let content = r#"[|f1/*0*/();|]
// @Filename: jalapeño.ts
export function f1() {}
export var v1 = 5;"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportFile0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
