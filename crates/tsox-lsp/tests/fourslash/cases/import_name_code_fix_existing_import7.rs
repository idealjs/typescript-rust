use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_existing_import7() {
    let content = r#"import [|{ v1 }|] from "../other_dir/module";
f1/*0*/();
// @Filename: ../other_dir/module.ts
export var v1 = 5;
export function f1();"#;
    let mut s = Session::new_for_test("importNameCodeFixExistingImport7", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
