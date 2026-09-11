use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_destructured_require1() {
    let content = r#"// @allowJs: true
// @Filename: util.js
class /*2*/Util {}
module.exports = { Util };
// @Filename: index.js
const { Util } = require('./util');
new [|Util/*1*/|]()"#;
    let mut s = Session::new_for_test("goToDefinitionDestructuredRequire1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
