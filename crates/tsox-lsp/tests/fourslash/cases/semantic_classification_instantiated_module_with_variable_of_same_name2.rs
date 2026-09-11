use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_classification_instantiated_module_with_variable_of_same_name2() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
}

module /*2*/M {
    var x = 10;
}

var /*3*/M = {
    foo: 10,
    bar: 20
}

var v: /*4*/M./*5*/I;

var x = /*6*/M;"#;
    let mut s = Session::new_for_test("semanticClassificationInstantiatedModuleWithVariableOfSameName2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
