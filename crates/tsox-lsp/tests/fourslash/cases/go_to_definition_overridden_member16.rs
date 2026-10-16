use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overridden_member16() {
    let content = r#"// @Filename: goToDefinitionOverrideJsdoc.ts
// @allowJs: true
// @checkJs: true
export class C extends CompletelyUndefined {
    /**
     * @override/*1*/
     * @returns {{}}
     */
    static foo() {
        return {}
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionOverriddenMember16", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
