use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classification_for_js_doc_template_tag() {
    let content = r#"/** @template T baring strait */
function ident<T>: T {
}"#;
    let mut s = Session::new_for_test("syntacticClassificationForJSDocTemplateTag", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
