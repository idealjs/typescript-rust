use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_export_as_default_existing_import() {
    let content = r#"import [|{ v1, v2, v3 }|] from "./module";
v4/*0*/();
// @Filename: module.ts
const v4 = 5;
export { v4 as default };
export const v1 = 5;
export const v2 = 5;
export const v3 = 5;"#;
    let _s = Session::new_for_test("importNameCodeFixExportAsDefaultExistingImport", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
