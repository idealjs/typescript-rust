use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_in_multi_line_comment() {
    let content = r#"// @Filename: justAComment.ts
/* /*0*/ */"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyNoJSDocCompletion(t, "0")
}
