use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_return3() {
    let content = r#"class C {
    /*end*/m() {
        [|/*start*/return|] 1;
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionReturn3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
