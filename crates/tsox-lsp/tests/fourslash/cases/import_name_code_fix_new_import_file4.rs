use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_file4() {
    let content = r#"[|let t: A/*0*/.B.I;|]
// @Filename: ./module.ts
export namespace A {
   export namespace B {
       export interface I { }
   }
}"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportFile4", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
