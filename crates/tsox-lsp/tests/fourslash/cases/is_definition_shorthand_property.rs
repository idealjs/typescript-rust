use tsox_lsp::fourslash::Session;


#[test]
fn is_definition_shorthand_property() {
    let content = r#"const /*1*/x = 1;
const y: { /*2*/x: number } = { /*3*/x };"#;
    let _s = Session::new_for_test("isDefinitionShorthandProperty", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
