use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_switch_case1() {
    let content = r#"switch (null ) {
  [|/*start*/case|] null: break;
}"#;
    let mut s = Session::new_for_test("goToDefinitionSwitchCase1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
