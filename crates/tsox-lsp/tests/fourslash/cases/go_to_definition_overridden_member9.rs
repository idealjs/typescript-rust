use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member9() {
    let content = r#"// @noImplicitOverride: true
interface I {
    m(): void;
}
class A {
    /*2*/m() {};
}
class B extends A implements I {
   [|/*1*/override|] m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember9", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
