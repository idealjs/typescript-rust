use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification_with_union_types() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
}

interface /*2*/I {
}
class /*3*/C {
}

var M: /*4*/M./*5*/I | /*6*/I | /*7*/C;
var I: typeof M | typeof /*8*/C;"#;
    let _s = Session::new_for_test("semanticClassificationWithUnionTypes", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
