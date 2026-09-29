use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_existing_import_equals0() {
    let content = r#"[|import ns = require("ambient-module");
var x = v1/*0*/ + 5;|]
// @Filename: ambientModule.ts
declare module "ambient-module" {
   export function f1();
   export var v1;
}"#;
    let _s = Session::new_for_test("importNameCodeFixExistingImportEquals0", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
