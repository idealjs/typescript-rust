use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_ambient0() {
    let content = r#"[|f1/*0*/();|]
// @Filename: ambientModule.ts
declare module "ambient-module" {
   export function f1();
   export var v1;
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportAmbient0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
