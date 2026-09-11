use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_with_multiple_js_doc_and_parameters() {
    let content = r#"/** */
/**
 * 
 * @param p 
 */
/** */
/*/**/
function foo(p) {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 7, `/**
}
