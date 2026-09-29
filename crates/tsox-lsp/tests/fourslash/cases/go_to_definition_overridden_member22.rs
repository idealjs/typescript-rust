use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member22() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop = "foo" as const;

abstract class A {}

export class B extends A {
  [|/*1*/override|] readonly [prop] = "B";
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember22", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
