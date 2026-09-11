use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_existing_import6() {
    let content = r#"import [|{ v1 }|] from "fake-module";
f1/*0*/();
// @Filename: ../package.json
{ "dependencies": { "fake-module": "latest" } }
// @Filename: ../node_modules/fake-module/index.ts
export var v1 = 5;
export function f1();"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport6", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
