use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_override_modifier18() {
    let content = r#"// @noImplicitOverride: true
class A {
    static foo() {}
}
class B extends A {
    [|static foo() {}|]
}"#;
    let mut s = Session::new_for_test("codeFixOverrideModifier18", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixAddOverrideModifier")
}
