use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_return2() {
    let content = r#"function foo() {
    return /*end*/() => {
        [|/*start*/return|] 10;
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionReturn2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
