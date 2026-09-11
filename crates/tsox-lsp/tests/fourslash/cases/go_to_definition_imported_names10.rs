use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names10() {
    let content = r#"// @allowjs: true
// @Filename: a.js
 class /*classDefinition*/Class {
   f;
 }
 module.exports.Class = Class;
// @Filename: b.js
const { Class } = require("./a");
 [|/*classAliasDefinition*/Class|];"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames10", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
