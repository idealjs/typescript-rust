use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_indented_identifier() {
    let content = r#"// @Filename: /a.ts
[|import * as b from "./b";
{
    x/**/
}|]
// @Filename: /b.ts
export const x = 0;"#;
    let _s = Session::new_for_test("importNameCodeFixIndentedIdentifier", content);
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
