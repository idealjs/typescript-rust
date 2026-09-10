use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_class_decl01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/*decl*/class C {
    private p;
    constructor(a, b, c, d);
    constructor(public a, private b, protected c, d, e?) {
    }

    foo();
    foo(a?, b?, ...args) {
    }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "decl", 3, `/** */`, nil)
}
