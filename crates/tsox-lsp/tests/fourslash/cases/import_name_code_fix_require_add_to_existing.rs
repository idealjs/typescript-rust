use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_require_add_to_existing() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: blah.js
export default class Blah {}
export const Named1 = 0;
export const Named2 = 1;
// @Filename: index.js
var path = require('path')
  , { promisify } = require('util')
  , { Named1 } = require('./blah')

new Blah"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "index.js");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
