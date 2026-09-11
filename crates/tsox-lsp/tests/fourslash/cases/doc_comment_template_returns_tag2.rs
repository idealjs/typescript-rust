use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_returns_tag2() {
    let content = r#"/*0*/
function f1(x: number, y: number) {
    return 1;
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 7, `/**
    // TODO: f.VerifyJSDocCompletion(t, "0", 7, `/**
}
