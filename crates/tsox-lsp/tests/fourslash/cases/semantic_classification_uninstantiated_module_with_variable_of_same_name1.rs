use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification_uninstantiated_module_with_variable_of_same_name1() {
    let content = r#"declare module /*0*/M {
    interface /*1*/I {

    }
}

var M = { I: 10 };"#;
    let _s = Session::new_for_test("semanticClassificationUninstantiatedModuleWithVariableOfSameName1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
