use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_with_multiple_js_doc1() {
    let content = r#"/** */
/*/**/
function foo() {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 3, `/** */`, nil)
}
