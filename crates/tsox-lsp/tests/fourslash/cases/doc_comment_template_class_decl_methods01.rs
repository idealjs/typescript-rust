use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_class_decl_methods01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class C {
/*0*/    /*1*/
    foo();
    /*2*/foo(a);
    /*3*/foo(a, b);
    /*4*/foo(a, {x: string}, [c]);
    /*5*/foo(a?, b?, ...args) {
    }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "0", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "1", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "2", 11, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "3", 11, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "4", 11, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "5", 11, `/**
}
