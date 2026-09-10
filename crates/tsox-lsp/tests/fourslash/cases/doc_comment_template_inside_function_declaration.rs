use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_inside_function_declaration() {
    let content = r#"// @Filename: functionDecl.ts
f/*0*/unction /*1*/foo/*2*/(/*3*/) /*4*/{ /*5*/}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: for _, marker := range f.Markers() {
}
