use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member5() {
    let content = r#"// @noImplicitOverride: true
class Foo extends (class {
    /*2*/m() {}
}) {
    [|/*1*/override|] m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember5", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
