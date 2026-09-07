use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_destructured_require1() {
    let content = r#"// @allowJs: true
// @Filename: util.js
class /*2*/Util {}
module.exports = { Util };
// @Filename: index.js
const { Util } = require('./util');
new [|Util/*1*/|]()"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
