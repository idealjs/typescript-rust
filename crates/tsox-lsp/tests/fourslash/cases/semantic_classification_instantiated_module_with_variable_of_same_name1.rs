use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
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
    let mut s = Session::new_for_test("semanticClassificationInstantiatedModuleWithVariableOfSameName1", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
