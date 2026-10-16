use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_require_umd() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: umd.d.ts
namespace Foo { function f() {} }
export = Foo;
export as namespace Foo;
// @Filename: index.js
Foo;
module.exports = {};"#;
    let mut s = Session::new_for_test("importNameCodeFix_require_UMD", content);
    fourslash::go_to_file(&mut s, "index.js");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
