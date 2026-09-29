use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_switch_case2() {
    let content = r#"switch (null) {
  [|/*start*/default|]: break;
}"#;
    let _s = Session::new_for_test("goToDefinitionSwitchCase2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
