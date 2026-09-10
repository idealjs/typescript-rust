use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member8() {
    let content = r#"// @noImplicitOverride: true
// @Filename: ./a.ts
export class A {
    /*2*/m() {}
}
// @Filename: ./b.ts
import { A } from "./a";
class B extends A {
    [|/*1*/override|] m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember8", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
