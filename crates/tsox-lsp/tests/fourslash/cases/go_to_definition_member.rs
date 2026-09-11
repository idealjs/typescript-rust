use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_member() {
    let content = r#"// @Filename: /a.ts
class A {
    private z/*z*/: string;
}"#;
    let mut s = Session::new_for_test("goToDefinitionMember", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "z")
}
