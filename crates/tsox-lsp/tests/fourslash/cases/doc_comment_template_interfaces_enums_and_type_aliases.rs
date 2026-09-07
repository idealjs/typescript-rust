use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_interfaces_enums_and_type_aliases() {
    let content = r#"/*interfaceFoo*/
interface Foo {
    /*propertybar*/
    bar: any;

    /*methodbaz*/
    baz(message: any): void;

    /*methodUnit*/
    unit(): void;
}

/*enumStatus*/
const enum Status {
    /*memberOpen*/
    Open,

    /*memberClosed*/
    Closed
}

/*aliasBar*/
type Bar = Foo & any;"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "interfaceFoo", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "propertybar", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "methodbaz", 11, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "methodUnit", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "enumStatus", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "memberOpen", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "memberClosed", 3, `/** */`, nil)
}
