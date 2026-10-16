use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_namespaces_and_modules02() {
    let content = r#"/*top*/
namespace n1.
    /*n2*/ n2.
    /*n3*/ n3 {
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "top", 3, `/** */`, nil)
    // TODO: f.VerifyNoJSDocCompletion(t, "n2")
    // TODO: f.VerifyNoJSDocCompletion(t, "n3")
}
