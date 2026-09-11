use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_return3() {
    let content = r#"class C {
    /*end*/m() {
        [|/*start*/return|] 1;
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionReturn3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
