use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member25() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop: symbol = Symbol();

abstract class A {}

export class B extends A {
  static [|/*1*/override|] [prop]() {}
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember25", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
