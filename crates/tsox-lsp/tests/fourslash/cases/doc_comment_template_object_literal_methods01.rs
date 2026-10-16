use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_object_literal_methods01() {
    let content = r#"var x = {
    /*0*/
    foo() {
        return undefined;
    }

    /*1*/
    [1 + 2 + 3 + Math.rand()](x: number, y: string, z = true) { }

    /*2*/
    m1: function(a) {}

    /*3*/
    m2: (a: string, b: string) => {}
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 11, `/**
    // TODO: f.VerifyJSDocCompletion(t, "1", 11, `/**
    // TODO: f.VerifyJSDocCompletion(t, "2", 11, `/**
    // TODO: f.VerifyJSDocCompletion(t, "3", 11, `/**
}
