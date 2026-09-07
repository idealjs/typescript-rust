use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member1() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	/*2*/p = '';
}
class Bar extends Foo {
	[|/*1*/override|] p = '';
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
