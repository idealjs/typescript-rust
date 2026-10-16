use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member7() {
    let content = r#"// @noImplicitOverride: true
class Foo {
    [|/*1*/override|] m() {}
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember7", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
