use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_with_multiple_js_doc2() {
    let content = r#"/** @typedef {string} Id */

/** /**/ */
function foo(x, y, z) {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 7, `/**
}
