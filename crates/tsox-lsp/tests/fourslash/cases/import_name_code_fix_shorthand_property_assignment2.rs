use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_shorthand_property_assignment2() {
    let content = r#"// @Filename: /a.ts
const a = 1;
export default a;
// @Filename: /b.ts
const b = { /**/a };"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
