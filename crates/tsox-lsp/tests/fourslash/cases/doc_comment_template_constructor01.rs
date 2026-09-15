use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_constructor01() {
    let content = r#"class C {
    private p;
    /*0*/
    constructor(a, b, c, d);
    /*1*/
    constructor(public a, private b, protected c, d, e?) {
    }

    foo();
    foo(a?, b?, ...args) {
    }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 11, `/**
    // TODO: f.VerifyJSDocCompletion(t, "1", 11, `/**
}
