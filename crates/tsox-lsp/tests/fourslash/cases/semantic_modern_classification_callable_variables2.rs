use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_callable_variables2() {
    let content = r#"import "node";
var fs = require("fs")
require.resolve('react');
require.resolve.paths;
interface LanguageMode { getFoldingRanges?: (d: string) => number[]; };
function (mode: LanguageMode | undefined) { if (mode && mode.getFoldingRanges) { return mode.getFoldingRanges('a'); }};
function b(a: () => void) { a(); };"#;
    let mut s = Session::new_for_test("semanticModernClassificationCallableVariables2", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
