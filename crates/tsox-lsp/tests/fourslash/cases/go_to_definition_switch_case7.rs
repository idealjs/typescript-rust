use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_switch_case7() {
    let content = r#"switch (null) {
  case null:
    export [|/*start*/default|] 123;"#;
    let _s = Session::new_for_test("goToDefinitionSwitchCase7", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
    // TODO: }
}
