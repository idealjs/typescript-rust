use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_function_expression() {
    let content = r#"/*above*/
const x = /*next*/ function f(p) {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: for _, marker := range f.MarkerNames() {
}
