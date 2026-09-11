use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_shorthand_property03() {
    let content = r#"var /*varDef*/x = {
    [|/*varProp*/x|]
}
let /*letDef*/y = {
    [|/*letProp*/y|]
}"#;
    let mut s = Session::new_for_test("goToDefinitionShorthandProperty03", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "varProp", "letProp")
}
