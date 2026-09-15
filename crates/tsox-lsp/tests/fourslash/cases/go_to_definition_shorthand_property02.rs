use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_shorthand_property02() {
    let content = r#"let x = {
    [|f/*1*/oo|]
}"#;
    let _s = Session::new_for_test("goToDefinitionShorthandProperty02", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
