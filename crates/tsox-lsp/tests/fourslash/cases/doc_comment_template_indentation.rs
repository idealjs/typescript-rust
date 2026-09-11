use tsox_lsp::fourslash::{self, Session};


#[test]
fn doc_comment_template_indentation() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: indents.ts
    a   /*2*/
    /*1*/
/*0*/        function foo() { }"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 3, `/** */`, nil)
    // TODO: f.VerifyJSDocCompletion(t, "1", 3, `/** */`, nil)
    // TODO: f.VerifyJSDocCompletion(t, "2", 3, `/** */`, nil)
}
