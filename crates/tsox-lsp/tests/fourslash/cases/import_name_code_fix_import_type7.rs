use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_import_type7() {
    let content = r#"// @module: es2015
// @Filename: /exports.ts
export interface SomeInterface {}
export class SomePig {}
// @Filename: /a.ts
import {
    type SomeInterface,
    type SomePig,
} from "./exports.js";
new SomePig/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType7", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
