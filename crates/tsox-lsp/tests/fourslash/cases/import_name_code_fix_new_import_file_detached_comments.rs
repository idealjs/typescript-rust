use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_file_detached_comments() {
    let content = r#"[|/**
 * This is a comment intended to be attached to this interface
 */
export interface SomeInterface {
}
f1/*0*/();|]
// @Filename: module.ts
export function f1() {}
export var v1 = 5;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
