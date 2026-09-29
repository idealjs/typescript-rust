use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("semanticModernClassificationCallableVariables", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
