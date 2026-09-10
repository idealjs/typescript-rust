use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
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
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixAddOverrideModifier")
}
