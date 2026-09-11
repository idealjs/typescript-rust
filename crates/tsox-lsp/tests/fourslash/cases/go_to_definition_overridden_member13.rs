use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member13() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	static /*2*/m() {}
}
class Bar extends Foo {
	static [|/*1*/override|] m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember13", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
