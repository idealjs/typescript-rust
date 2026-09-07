use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member15() {
    let content = r#"// @noImplicitOverride: true
class A {
    static /*2*/m() {}
}
class B extends A {}
class C extends B {
    static [|/*1*/override|] m() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
