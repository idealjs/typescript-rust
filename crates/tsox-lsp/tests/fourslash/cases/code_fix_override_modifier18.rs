use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_override_modifier18() {
    let content = r#"// @noImplicitOverride: true
class A {
    static foo() {}
}
class B extends A {
    [|static foo() {}|]
}"#;
    let _s = Session::new_for_test("codeFixOverrideModifier18", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixAddOverrideModifier")
}
