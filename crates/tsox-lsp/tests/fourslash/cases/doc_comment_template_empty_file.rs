use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_empty_file() {
    let content = r#"// @Filename: emptyFile.ts
/*0*/"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyNoJSDocCompletion(t, "0")
}
