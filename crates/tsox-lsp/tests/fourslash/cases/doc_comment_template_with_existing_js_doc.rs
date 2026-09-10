use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_with_existing_js_doc() {
    let content = r#"/** /**/ */

/**
 * @param {string} a
 * @param {string} b
 */
function foo(a, b) {
    return a + b;
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyNoJSDocCompletion"); // f.VerifyNoJSDocCompletion(t, "")
}
