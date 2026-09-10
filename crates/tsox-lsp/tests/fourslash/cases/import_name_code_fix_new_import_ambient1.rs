use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_ambient1() {
    let content = r#"import d from "other-ambient-module";
import * as ns from "yet-another-ambient-module";
var x = v1/*0*/ + 5;
// @Filename: ambientModule.ts
declare module "ambient-module" {
   export function f1();
   export var v1;
}
// @Filename: otherAmbientModule.ts
declare module "other-ambient-module" {
   export default function f2();
}
// @Filename: yetAnotherAmbientModule.ts
declare module "yet-another-ambient-module" {
   export function f3();
   export var v3;
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportAmbient1", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
