use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_import_type5() {
    let content = r#"// @module: es2015
// @Filename: /exports.ts
export interface SomeInterface {}
export class SomePig {}
// @Filename: /a.ts
import type { SomeInterface, SomePig } from "./exports.js";
new SomePig/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType5", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
