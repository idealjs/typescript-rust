use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_file2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"[|f1/*0*/();|]
// @Filename: ../../other_dir/module.ts
export var v1 = 5;
export function f1();"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFile2", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
