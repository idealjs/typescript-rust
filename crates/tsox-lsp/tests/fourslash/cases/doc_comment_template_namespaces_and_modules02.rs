use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_namespaces_and_modules02() {
    let content = r#"/*top*/
namespace n1.
    /*n2*/ n2.
    /*n3*/ n3 {
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "top", 3, `/** */`, nil)
    fourslash::unsupported("VerifyNoJSDocCompletion"); // f.VerifyNoJSDocCompletion(t, "n2")
    fourslash::unsupported("VerifyNoJSDocCompletion"); // f.VerifyNoJSDocCompletion(t, "n3")
}
