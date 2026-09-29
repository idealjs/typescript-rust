use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_js_require_destructuring1() {
    let content = r#"// @allowJs: true
// @noEmit: true
// @checkJs: true
// @Filename: /X.js
module.exports = { x: 1 };
// @Filename: /Y.js
const { /*1*/x: { y } } = require("./X");"#;
    let _s = Session::new_for_test("findAllReferencesJsRequireDestructuring1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
