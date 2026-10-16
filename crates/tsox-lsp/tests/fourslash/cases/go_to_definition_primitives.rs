use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_primitives() {
    let content = r#"var x: st/*primitive*/ring;"#;
    let _s = Session::new_for_test("goToDefinitionPrimitives", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "primitive")
}
