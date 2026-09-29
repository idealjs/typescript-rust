use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_fix_with_multiple_module_export_assignment() {
    let content = r#"// @module: esnext
// @allowJs: true
// @checkJs: true
// @Filename: /a.js
function f() {}
module.exports = f;
module.exports = 42;
// @Filename: /b.js
export const foo = 0;
// @Filename: /c.js
foo"#;
    let mut s = Session::new_for_test("importFixWithMultipleModuleExportAssignment", content);
    fourslash::go_to_file(&mut s, "/c.js");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
