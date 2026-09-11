use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_yield3() {
    let content = r#"class C {
    notAGenerator() {
      [|/*start1*/yield|] 0;
    }

    foo*/*end2*/() {
      [|/*start2*/yield|] 0;
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionYield3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
