use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_require() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: foo.js
module.exports = function foo() {}
// @Filename: utils.js
function util1() {}
function util2() {}
module.exports = { util1, util2 };
// @Filename: blah.js
export default class Blah {}
// @Filename: index.js
foo();
util1();
util2();
new Blah;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "index.js");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
