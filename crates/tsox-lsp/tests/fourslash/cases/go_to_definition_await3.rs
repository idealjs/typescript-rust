use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_await3() {
    let content = r#"class C {
    notAsync() {
      [|/*start1*/await|] Promise.resolve(0);
    }

    async /*end2*/foo() {
      [|/*start2*/await|] Promise.resolve(0);
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionAwait3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
