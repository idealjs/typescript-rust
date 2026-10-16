use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_class_decl01() {
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
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "decl", 3, `/** */`, nil)
}
