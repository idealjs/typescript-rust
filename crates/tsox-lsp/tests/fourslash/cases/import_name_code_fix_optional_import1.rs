use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_optional_import1() {
    let content = r#"// @Filename: a/f1.ts
[|foo/*0*/();|]
// @Filename: a/node_modules/bar/index.ts
export function foo() {};
// @Filename: a/foo.ts
export { foo } from "bar";"#;
    let _s = Session::new_for_test("importNameCodeFixOptionalImport1", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
