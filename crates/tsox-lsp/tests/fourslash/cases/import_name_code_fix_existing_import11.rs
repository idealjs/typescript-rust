use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_existing_import11() {
    let content = r#"import [|{
    v1, v2,
    v3
}|] from "./module";
f1/*0*/();
// @Filename: module.ts
 export function f1() {}
 export var v1 = 5;
 export var v2 = 5;
 export var v3 = 5;"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport11", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
