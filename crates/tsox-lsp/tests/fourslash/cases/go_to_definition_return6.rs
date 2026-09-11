use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_return6() {
    let content = r#"function foo() {
    return /*end*/function () {
        [|/*start*/return|] 10;
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionReturn6", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
