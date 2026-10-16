use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_await2() {
    let content = r#"[|/*start*/await|] Promise.resolve(0);"#;
    let _s = Session::new_for_test("goToDefinitionAwait2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
