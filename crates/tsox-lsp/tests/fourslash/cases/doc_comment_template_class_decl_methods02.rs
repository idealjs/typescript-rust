use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_class_decl_methods02() {
    let content = r#"class C {
    /*0*/
    [Symbol.iterator]() {
        return undefined;
    }
    /*1*/
    [1 + 2 + 3 + Math.rand()](x: number, y: string, z = true) { }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "0", 11, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "1", 11, `/**
}
