use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_js_module_name() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
/*2*/module.exports = {};
// @Filename: bar.js
var x = require([|/*1*/"./foo"|]);"#;
    let mut s = Session::new_for_test("goToDefinitionJsModuleName", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
