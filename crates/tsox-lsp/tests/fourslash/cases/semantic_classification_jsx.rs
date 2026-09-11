use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_classification_jsx() {
    let content = r#"// @Filename: /a.tsx
const Component = () => <div>Hello</div>;
const afterJSX = 42;
const alsoAfterJSX = "test";"#;
    let mut s = Session::new_for_test("semanticClassificationJSX", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
