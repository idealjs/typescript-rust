use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_property_assignment() {
    let content = r#"export const /*FunctionResult*/Component = () => { return "OK"}
Component./*PropertyResult*/displayName = 'Component'

[|/*FunctionClick*/Component|]

Component.[|/*PropertyClick*/displayName|]"#;
    let _s = Session::new_for_test("goToDefinitionPropertyAssignment", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "FunctionClick", "PropertyClick")
}
