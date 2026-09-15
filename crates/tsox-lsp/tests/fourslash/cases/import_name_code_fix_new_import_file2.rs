use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_new_import_file2() {
    let content = r#"[|f1/*0*/();|]
// @Filename: ../../other_dir/module.ts
export var v1 = 5;
export function f1();"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportFile2", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
