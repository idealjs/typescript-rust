use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names8() {
    let content = r#"// @allowjs: true
// @Filename: b.js
import { [|/*classAliasDefinition*/Class|] } from "./a";
// @Filename: a.js
class /*classDefinition*/Class {
    private f;
}
 export { Class };"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames8", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
