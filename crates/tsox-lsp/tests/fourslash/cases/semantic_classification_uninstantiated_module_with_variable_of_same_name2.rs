use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_classification_uninstantiated_module_with_variable_of_same_name2() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
}

var /*2*/M = {
    foo: 10,
    bar: 20
}

var v: /*3*/M./*4*/I;

var x = /*5*/M;"#;
    let mut s = Session::new_for_test("semanticClassificationUninstantiatedModuleWithVariableOfSameName2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
