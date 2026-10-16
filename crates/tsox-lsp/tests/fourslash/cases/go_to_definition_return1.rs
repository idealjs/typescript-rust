use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_return1() {
    let content = r#"function /*end*/foo() {
    [|/*start*/return|] 10;
}"#;
    let _s = Session::new_for_test("goToDefinitionReturn1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
