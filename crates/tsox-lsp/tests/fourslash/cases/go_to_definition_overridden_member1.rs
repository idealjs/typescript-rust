use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member1() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	/*2*/p = '';
}
class Bar extends Foo {
	[|/*1*/override|] p = '';
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
