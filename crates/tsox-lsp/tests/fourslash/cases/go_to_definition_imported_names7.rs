use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names7() {
    let content = r#"// @Filename: b.ts
import [|/*classAliasDefinition*/defaultExport|] from "./a";
// @Filename: a.ts
class /*classDefinition*/Class {
    private f;
}
export default Class;"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames7", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
