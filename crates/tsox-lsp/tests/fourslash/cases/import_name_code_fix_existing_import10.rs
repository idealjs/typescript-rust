use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_existing_import10() {
    let content = r#"import [|{
    v1,
    v2
}|] from "./module";
f1/*0*/();
// @Filename: module.ts
export function f1() {}
export var v1 = 5;
export var v2 = 5;
export var v3 = 5;"#;
    let _s = Session::new_for_test("importNameCodeFixExistingImport10", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
