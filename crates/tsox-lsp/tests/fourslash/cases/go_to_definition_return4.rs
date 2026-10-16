use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_return4() {
    let content = r#"[|/*start*/return|];"#;
    let _s = Session::new_for_test("goToDefinitionReturn4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
