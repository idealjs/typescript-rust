use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_imported_names2() {
    let content = r#"// @Filename: b.ts
import {[|/*classAliasDefinition*/Class|]} from "./a";
// @Filename: a.ts
export namespace Module {
}
export class /*classDefinition*/Class {
    private f;
}
export interface Interface {
    x;
}"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
