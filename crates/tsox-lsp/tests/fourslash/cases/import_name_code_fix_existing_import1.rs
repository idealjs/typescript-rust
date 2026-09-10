use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_existing_import1() {
    let content = r#"import d, [|{ v1 }|] from "./module";
f1/*0*/();
// @Filename: module.ts
export function f1() {}
export var v1 = 5;
export default var d1 = 6;"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport1", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
