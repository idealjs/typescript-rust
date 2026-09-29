use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export() {
    let content = r#"// @module: esnext
// @allowJs: true
// @checkJs: true
// @Filename: /a.js
class C {}
export default C;
// @Filename: /b.js
[|C;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_defaultExport", content);
    fourslash::go_to_file(&mut s, "/b.js");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
