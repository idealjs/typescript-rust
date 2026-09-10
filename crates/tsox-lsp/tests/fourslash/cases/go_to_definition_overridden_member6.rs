use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member6() {
    let content = r#"// @noImplicitOverride: true
class Foo {
    m() {}
}
class Bar extends Foo {
    [|/*1*/override|] m1() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember6", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
