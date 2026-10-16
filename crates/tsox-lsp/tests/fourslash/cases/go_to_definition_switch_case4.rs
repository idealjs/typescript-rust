use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_switch_case4() {
    let content = r#"     switch (null) {
         case null: break;
     }

     switch (null) {
        [|/*start*/case|] null: break;
     }"#;
    let _s = Session::new_for_test("goToDefinitionSwitchCase4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
