use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_callable_variables2() {
    let content = r#"import "node";
var fs = require("fs")
require.resolve('react');
require.resolve.paths;
interface LanguageMode { getFoldingRanges?: (d: string) => number[]; };
function (mode: LanguageMode | undefined) { if (mode && mode.getFoldingRanges) { return mode.getFoldingRanges('a'); }};
function b(a: () => void) { a(); };"#;
    let _s = Session::new_for_test("semanticModernClassificationCallableVariables2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
