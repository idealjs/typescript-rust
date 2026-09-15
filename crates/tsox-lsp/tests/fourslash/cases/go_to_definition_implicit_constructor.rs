use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_implicit_constructor() {
    let content = r#"class /*constructorDefinition*/ImplicitConstructor {
}
var implicitConstructor = new /*constructorReference*/ImplicitConstructor();"#;
    let _s = Session::new_for_test("goToDefinitionImplicitConstructor", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "constructorReference")
}
