use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_variable_statements03() {
    let content = r#"/*a*/
var a = x => x

/*b*/
let b = (x,y,z) => x + y + z;

/*c*/
const c = ((x => +x))

/*d*/
let d = (function () { })

/*e*/
let e = function e([a,b,c]) {
    return "hello"
};

/*f*/
let f = class {
}

/*g*/
const g = ((class G {
    constructor(private x);
    constructor(x,y,z);
    constructor(x,y,z, ...okayThatsEnough) {
    }
}))"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "a", 7, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "b", 7, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "c", 7, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "d", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "e", 7, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "f", 3, `/** */`, nil)
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "g", 7, `/**
}
