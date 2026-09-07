use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member12() {
    let content = r#"// @noImplicitOverride: true
class Foo {
	static /*2*/p = '';
}
class Bar extends Foo {
	static [|/*1*/override|] p = '';
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
