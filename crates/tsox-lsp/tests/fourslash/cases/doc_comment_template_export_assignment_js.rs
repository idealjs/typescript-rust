use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_export_assignment_js() {
    let content = r#"// @allowJs: true
// @allowJs: true
// @checkJs: true
// @Filename: index.js
/** /**/ */
exports.foo = (a) => {};"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 7, `/**
}
