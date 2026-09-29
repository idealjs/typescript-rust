use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_destructured_require2() {
    let content = r#"// @allowJs: true
// @Filename: util.js
class /*2*/Util {}
module.exports = { Util };
// @Filename: reexport.js
const { Util } = require('./util');
module.exports = { Util };
// @Filename: index.js
const { Util } = require('./reexport');
new [|Util/*1*/|]()"#;
    let _s = Session::new_for_test("goToDefinitionDestructuredRequire2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
