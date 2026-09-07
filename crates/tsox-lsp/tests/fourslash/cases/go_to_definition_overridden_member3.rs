use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member3() {
    let content = r#"// @noImplicitOverride: true
abstract class Foo {
	abstract /*2*/m() {}
}

export class Bar extends Foo {
	[|/*1*/override|] m() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
