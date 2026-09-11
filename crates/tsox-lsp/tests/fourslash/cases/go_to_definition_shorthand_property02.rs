use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_shorthand_property02() {
    let content = r#"let x = {
    [|f/*1*/oo|]
}"#;
    let mut s = Session::new_for_test("goToDefinitionShorthandProperty02", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
