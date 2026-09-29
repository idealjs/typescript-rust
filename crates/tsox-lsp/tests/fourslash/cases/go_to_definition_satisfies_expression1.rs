use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_satisfies_expression1() {
    let content = r#"const STRINGS = {
    [|/*definition*/title|]: 'A Title',
} satisfies Record<string,string>;

//somewhere in app
STRINGS.[|/*usage*/title|]"#;
    let _s = Session::new_for_test("goToDefinitionSatisfiesExpression1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "definition", "usage")
}
