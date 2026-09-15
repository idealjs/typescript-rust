use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_imported_names11() {
    let content = r#"// @allowjs: true
// @Filename: a.js
 class /*classDefinition*/Class {
     f;
 }
 module.exports = { Class };
// @Filename: b.js
const { Class } = require("./a");
 [|/*classAliasDefinition*/Class|];"#;
    let _s = Session::new_for_test("goToDefinitionImportedNames11", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
