use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_imported_names4() {
    let content = r#"// @Filename: b.ts
import {Class as [|/*classAliasDefinition*/ClassAlias|]} from "./a";
// @Filename: a.ts
export namespace Module {
}
export class /*classDefinition*/Class {
    private f;
}
export interface Interface {
    x;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "classAliasDefinition")
}
