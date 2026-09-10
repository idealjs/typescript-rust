use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_re_export_default() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /user.ts
foo;
// @Filename: /user2.ts
unnamed;
// @Filename: /user3.ts
reExportUnnamed;
// @Filename: /reExportNamed.ts
export { default } from "./named";
// @Filename: /reExportUnnamed.ts
export { default } from "./unnamed";
// @Filename: /named.ts
function foo() {}
export default foo;
// @Filename: /unnamed.ts
export default 0;"#;
    let mut s = Session::new_for_test("importNameCodeFix_reExportDefault", content);
    fourslash::go_to_file(&mut s, "/user.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/user2.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/user3.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
