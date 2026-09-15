use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_on_invalid_parameter_decorator() {
    let content = r#"function f(@/*1*/f) {}"#;
    let _s = Session::new_for_test("goToDefinitionOnInvalidParameterDecorator", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
