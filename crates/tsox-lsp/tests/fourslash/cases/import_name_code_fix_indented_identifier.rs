use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_indented_identifier() {
    let content = r#"// @Filename: /a.ts
[|import * as b from "./b";
{
    x/**/
}|]
// @Filename: /b.ts
export const x = 0;"#;
    let mut s = Session::new_for_test("importNameCodeFixIndentedIdentifier", content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
