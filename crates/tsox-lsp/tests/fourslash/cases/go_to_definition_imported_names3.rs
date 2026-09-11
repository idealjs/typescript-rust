use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_imported_names3() {
    let content = r#"// @Filename: e.ts
 import {M, [|/*classAliasDefinition*/C|], I} from "./d";
 var c = new [|/*classReference*/C|]();
// @Filename: d.ts
export * from "./c";
// @Filename: c.ts
export {Module as M, Class as C, Interface as I} from "./b";
// @Filename: b.ts
export * from "./a";
// @Filename: a.ts
export namespace Module {
}
export class /*classDefinition*/Class {
    private f;
}
export interface Interface {
    x;
}"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "classReference", "classAliasDefinition")
}
