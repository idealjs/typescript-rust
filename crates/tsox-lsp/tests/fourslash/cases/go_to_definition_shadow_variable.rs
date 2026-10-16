use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shadow_variable() {
    let content = r#"var shadowVariable = "foo";
function shadowVariableTestModule() {
    var /*shadowVariableDefinition*/shadowVariable;
    /*shadowVariableReference*/shadowVariable = 1;
}"#;
    let _s = Session::new_for_test("goToDefinitionShadowVariable", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "shadowVariableReference")
}
