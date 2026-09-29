use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_regex() {
    let content = r#"var regex = /*0*///*1*/asdf/*2*/ /*3*///*4*/;"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: for _, marker := range f.Markers() {
}
