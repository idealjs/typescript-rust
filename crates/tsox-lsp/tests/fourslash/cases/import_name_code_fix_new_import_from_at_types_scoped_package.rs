use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_from_at_types_scoped_package() {
    let content = r#"[|f1/*0*/();|]
// @Filename: node_modules/@types/myLib__scoped/index.d.ts
export function f1() {}
export var v1 = 5;"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportFromAtTypesScopedPackage", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
