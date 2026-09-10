use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_default_export4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /foo.ts
const a = () => {};
export default a;
// @Filename: /test.ts
[|foo|];"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport4", content);
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
