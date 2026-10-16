use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member12() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	static /*2*/p = '';
}
class Bar extends Foo {
	static [|/*1*/override|] p = '';
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember12", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
