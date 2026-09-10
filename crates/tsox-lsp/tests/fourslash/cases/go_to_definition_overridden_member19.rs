use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member19() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop = "foo" as const;

abstract class A {
  static readonly /*2*/[prop] = "A";
}

export class B extends A {
  static [|/*1*/override|] readonly [prop] = "B";
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember19", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
