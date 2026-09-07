use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_variable_statements01() {
    let content = r#"/*a*/
var a = 10;

/*b*/
let b = "";

/*c*/
const c = 30;

/*d*/
let d = {
    foo: 10,
    bar: "20"
};

/*e*/
let e = function e(x, y, z) {
    return +(x + y + z);
};

/*f*/
let f = class F {
    constructor(a, b, c) {
        this.a = a;
        this.b = b || (this.c = c);
    }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: for _, varName := range []string{"a", "b", "c", "d"} {
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "e", 7, `/**
    fourslash::unsupported("VerifyJSDocCompletion"); // f.VerifyJSDocCompletion(t, "f", 7, `/**
}
