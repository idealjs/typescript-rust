use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_type_predicate() {
    let content = r#"class /*classDeclaration*/A {}
function f(/*parameterDeclaration*/parameter: any): [|/*parameterName*/parameter|] is [|/*typeReference*/A|] {
    return typeof parameter === "string";
}"#;
    let _s = Session::new_for_test("goToDefinitionTypePredicate", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "parameterName", "typeReference")
}
