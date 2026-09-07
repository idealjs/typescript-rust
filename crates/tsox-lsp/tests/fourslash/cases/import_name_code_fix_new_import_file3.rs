use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_file3() {
    let content = r#"[|let t: XXX/*0*/.I;|]
// @Filename: ./module.ts
export namespace XXX {
   export interface I {
   }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
