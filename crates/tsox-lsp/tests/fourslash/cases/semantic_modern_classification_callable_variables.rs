use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_callable_variables() {
    let content = r#"class A { onEvent: () => void; }
const x = new A().onEvent;
const match = (s: any) => x();
const other = match;
match({ other });
interface B = { (): string; }; var b: B
var s: String;
var t: { (): string; foo: string};"#;
    let mut s = Session::new_for_test("semanticModernClassificationCallableVariables", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
