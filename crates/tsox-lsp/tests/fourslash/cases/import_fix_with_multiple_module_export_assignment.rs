use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/c.js");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
