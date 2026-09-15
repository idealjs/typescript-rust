use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_file3() {
    let content = r#"[|let t: XXX/*0*/.I;|]
// @Filename: ./module.ts
export namespace XXX {
   export interface I {
   }
}"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportFile3", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
