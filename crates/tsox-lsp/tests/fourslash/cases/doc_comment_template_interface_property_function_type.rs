use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_interface_property_function_type() {
    let content = r#"interface I {
    /**/
    foo: (a: number, b: string) => void;
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "", 11, `/**
    // TODO: f.VerifyJSDocCompletion(t, "", 11, `/**
}
