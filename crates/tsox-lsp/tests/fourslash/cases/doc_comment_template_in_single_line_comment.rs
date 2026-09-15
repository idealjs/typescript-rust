use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_in_single_line_comment() {
    let content = r#"// @Filename: justAComment.ts
// We want to check off-by-one errors in assessing the end of the comment, so we check twice,
// first with a trailing space and then without.
// /*0*/ 
// /*1*/
// We also want to check EOF handling at the end of a comment
// /*2*/"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: for _, marker := range f.Markers() {
}
