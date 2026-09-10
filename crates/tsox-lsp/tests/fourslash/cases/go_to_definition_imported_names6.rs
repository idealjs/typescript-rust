use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_imported_names6() {
    let content = r#"// @Filename: b.ts
import [|/*moduleAliasDefinition*/alias|] = require("./a");
// @Filename: a.ts
/*moduleDefinition*/export namespace Module {
}
export class Class {
    private f;
}
export interface Interface {
    x;
}"#;
    let mut s = Session::new_for_test("goToDefinitionImportedNames6", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "moduleAliasDefinition")
}
