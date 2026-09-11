use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names9() {
    let content = r#"// @allowjs: true
// @Filename: a.js
class /*classDefinition*/Class {
    f;
}
 export { Class };
// @Filename: b.js
const { Class } = require("./a");
 [|/*classAliasDefinition*/Class|];"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames9", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
