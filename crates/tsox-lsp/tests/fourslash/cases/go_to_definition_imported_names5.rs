use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names5() {
    let content = r#"// @Filename: b.ts
export {Class as [|/*classAliasDefinition*/ClassAlias|]} from "./a";
// @Filename: a.ts
export namespace Module {
}
export class /*classDefinition*/Class {
    private f;
}
export interface Interface {
    x;
}"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
