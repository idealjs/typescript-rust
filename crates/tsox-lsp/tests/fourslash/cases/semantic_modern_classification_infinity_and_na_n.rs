use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
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
    let mut s = Session::new_for_test("semanticModernClassificationInfinityAndNaN", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
