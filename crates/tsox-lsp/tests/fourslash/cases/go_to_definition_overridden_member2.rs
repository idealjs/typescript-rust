use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member2() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	/*2*/m() {}
}

class Bar extends Foo {
	[|/*1*/override|] m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
