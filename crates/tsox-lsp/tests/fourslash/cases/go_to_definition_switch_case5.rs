use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_switch_case5() {
    let content = r#"export [|/*start*/default|] {}"#;
    let _s = Session::new_for_test("goToDefinitionSwitchCase5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
