use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_namespaces_and_modules01() {
    let content = r#"/*namespaceN*/
namespace n {
}

/*namespaceM*/
namespace m {
}

/*ambientModule*/
module "ambientModule" {
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "namespaceN", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "namespaceM", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "ambientModule", 3, `/** */`, nil)
}
