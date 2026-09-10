use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member26() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop: symbol = Symbol();

abstract class A {}

export class B extends A {
  [|/*1*/override|] [prop]() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember26", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
