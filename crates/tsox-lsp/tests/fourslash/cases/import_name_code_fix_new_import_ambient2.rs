use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_ambient2() {
    let content = r#"[|/*!
 * I'm a license or something
 */
f1/*0*/();|]
// @Filename: ambientModule.ts
 declare module "ambient-module" {
    export function f1();
    export var v1;
 }"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportAmbient2", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
