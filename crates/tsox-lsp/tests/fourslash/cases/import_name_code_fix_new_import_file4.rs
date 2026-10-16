use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_new_import_file4() {
    let content = r#"[|let t: A/*0*/.B.I;|]
// @Filename: ./module.ts
export namespace A {
   export namespace B {
       export interface I { }
   }
}"#;
    let _s = Session::new_for_test("importNameCodeFixNewImportFile4", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
