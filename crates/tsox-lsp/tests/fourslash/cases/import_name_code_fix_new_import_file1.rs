use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_file1() {
    let content = r#"[|/// <reference path="./tripleSlashReference.ts" />
f1/*0*/();|]
// @Filename: Module.ts
export function f1() {}
export var v1 = 5;
// @Filename: tripleSlashReference.ts
var x = 5;/*dummy*/"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFile1", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
