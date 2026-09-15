use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shadow_variable_inside_module() {
    let content = r#"namespace shdModule {
    var /*shadowVariableDefinition*/shdVar;
    /*shadowVariableReference*/shdVar = 1;
}"#;
    let _s = Session::new_for_test("goToDefinitionShadowVariableInsideModule", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "shadowVariableReference")
}
