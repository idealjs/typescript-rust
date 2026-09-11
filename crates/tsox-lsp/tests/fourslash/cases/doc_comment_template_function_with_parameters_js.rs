use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_function_with_parameters_js() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/*0*/
function f(a, ...b): boolean {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 7, `/**
}
