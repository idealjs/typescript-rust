use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn go_to_definition_switch_case7() {
    let content = r#"switch (null) {
  case null:
    export [|/*start*/default|] 123;"#;
    let mut s = Session::new_for_test("goToDefinitionSwitchCase7", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
    // TODO: }
}
