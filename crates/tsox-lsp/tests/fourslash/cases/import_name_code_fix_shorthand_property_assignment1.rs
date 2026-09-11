use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_shorthand_property_assignment1() {
    let content = r#"// @Filename: /a.ts
export const a = 1;
// @Filename: /b.ts
const b = { /**/a };"#;
    let mut s = Session::new_for_test("importNameCodeFix_shorthandPropertyAssignment1", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
