use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_import_type4() {
    let content = r#"// @preserveValueImports: true
// @isolatedModules: true
// @module: es2015
// @Filename: /exports.ts
export interface SomeInterface {}
export class SomePig {}
// @Filename: /a.ts
import type { SomeInterface } from "./exports.js";
new SomePig/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType4", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
