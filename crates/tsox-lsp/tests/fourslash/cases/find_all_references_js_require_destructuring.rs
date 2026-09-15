use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_js_require_destructuring() {
    let content = r#"// @allowJs: true
// @noEmit: true
// @checkJs: true
// @Filename: foo.js
module.exports = {
    foo: '1'
};
// @Filename: bar.js
const { /*1*/foo: bar } = require('./foo');"#;
    let _s = Session::new_for_test("findAllReferencesJsRequireDestructuring", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
