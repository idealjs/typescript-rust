use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_commonjs_allow_synthetic() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @allowJs: true
// @checkJs: true
// @allowSyntheticDefaultImports: true
// @Filename: /test_module.js
const MY_EXPORTS = {}
module.exports = MY_EXPORTS;
// @Filename: /index.js
const newVar = {
  any: MY_EXPORTS/**/,
}"#;
    let mut s = Session::new_for_test("importNameCodeFix_commonjs_allowSynthetic", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
