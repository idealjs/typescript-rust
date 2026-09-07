use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
