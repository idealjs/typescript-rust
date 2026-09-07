use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_existing_import_equals0() {
    let content = r#"[|import ns = require("ambient-module");
var x = v1/*0*/ + 5;|]
// @Filename: ambientModule.ts
declare module "ambient-module" {
   export function f1();
   export var v1;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
