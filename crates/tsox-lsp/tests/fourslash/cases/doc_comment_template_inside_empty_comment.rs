use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_inside_empty_comment() {
    let content = r#"/** /**/ */
function f(p) { return p; }

/** Doc/*1*/ */
function g(p) { return p; }"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 7, `/**
    // TODO: f.VerifyNoJSDocCompletion(t, "1")
}
