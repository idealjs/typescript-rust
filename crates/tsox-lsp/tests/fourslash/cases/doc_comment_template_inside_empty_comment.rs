use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_inside_empty_comment() {
    let content = r#"/** /**/ */
function f(p) { return p; }

/** Doc/*1*/ */
function g(p) { return p; }"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "", 7, `/**
    fourslash::unsupported("VerifyNoJSDocCompletion"); // f.VerifyNoJSDocCompletion(t, "1")
}
