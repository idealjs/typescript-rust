use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_require_import_vs_require_module_target() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @module: es2015
// @Filename: a.js
export const x = 0;
// @Filename: index.js
x"#;
    let mut s = Session::new_for_test("importNameCodeFix_require_importVsRequire_moduleTarget", content);
    fourslash::go_to_file(&mut s, "index.js");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    fourslash::go_to_position(&mut s, 0);
    fourslash::insert_line(&mut s, "const fs = require('fs');\n");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
