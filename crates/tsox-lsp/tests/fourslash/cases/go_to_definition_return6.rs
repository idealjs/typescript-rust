use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_return6() {
    let content = r#"function foo() {
    return /*end*/function () {
        [|/*start*/return|] 10;
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionReturn6", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
