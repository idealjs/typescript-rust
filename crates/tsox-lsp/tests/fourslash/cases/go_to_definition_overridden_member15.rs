use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("goToDefinitionOverriddenMember15", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
