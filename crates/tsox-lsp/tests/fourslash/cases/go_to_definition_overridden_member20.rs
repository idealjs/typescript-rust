use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member20() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop = "foo" as const;

abstract class A {
  readonly /*2*/[prop] = "A";
}

export class B extends A {
  [|/*1*/override|] readonly [prop] = "B";
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember20", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
