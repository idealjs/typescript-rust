use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_switch_case3() {
    let content = r#"switch (null) {
  [|/*start1*/default|]: {
    switch (null) {
      [|/*start2*/default|]: break;
    }
  };
}"#;
    let mut s = Session::new_for_test("goToDefinitionSwitchCase3", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
