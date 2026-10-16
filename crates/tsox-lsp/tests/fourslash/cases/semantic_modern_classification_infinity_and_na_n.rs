use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_infinity_and_na_n() {
    let content = r#" Infinity;
 NaN;

// Regular properties

const obj1 = {
    Infinity: 100,
    NaN: 200,
    "-Infinity": 300
};

obj1.Infinity;
obj1.NaN;
obj1["-Infinity"];

// Shorthand properties

const obj2 = {
    Infinity,
    NaN,
}

obj2.Infinity;
obj2.NaN;"#;
    let _s = Session::new_for_test("semanticModernClassificationInfinityAndNaN", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
