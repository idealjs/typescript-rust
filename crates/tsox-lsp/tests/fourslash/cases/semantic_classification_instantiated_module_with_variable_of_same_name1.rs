use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification_instantiated_module_with_variable_of_same_name1() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
    var x = 10;
}

var /*2*/M = {
    foo: 10,
    bar: 20
}

var v: /*3*/M./*4*/I;

var x = /*5*/M;"#;
    let _s = Session::new_for_test("semanticClassificationInstantiatedModuleWithVariableOfSameName1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
