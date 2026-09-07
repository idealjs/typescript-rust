use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_existing_import0() {
    let content = r#"import [|{ v1 }|] from "./module";
f1/*0*/();
// @Filename: module.ts
export function f1() {}
export var v1 = 5;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
