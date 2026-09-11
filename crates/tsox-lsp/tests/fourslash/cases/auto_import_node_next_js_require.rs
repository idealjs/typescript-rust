use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_node_next_js_require() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: node18
// @allowJs: true
// @checkJs: true
// @noEmit: true
// @Filename: /matrix.js
exports.variants = [];
// @Filename: /main.js
exports.dedupeLines = data => {
  variants/**/
}
// @Filename: /totally-irrelevant-no-way-this-changes-things-right.js
export default 0;"#;
    let mut s = Session::new_for_test("autoImportNodeNextJSRequire", content);
    fourslash::go_to_file(&mut s, "/main.js");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
