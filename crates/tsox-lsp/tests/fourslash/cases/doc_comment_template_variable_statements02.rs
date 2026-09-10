use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: capabilities := fourslash.GetDefaultCapabilities()"]
#[test]
fn doc_comment_template_variable_statements02() {
    let content = r#"/*a*/
var a1 = 10, a2 = 20;

/*b*/
let b1 = "", b2 = true;

/*c*/
const c1 = 30, c2 = 40;

/*d*/
let d1 = function d(x, y, z) {
    return +(x + y + z);
}, d2 = 50;

/*e*/
let e1 = class E {
    constructor(a, b, c) {
        this.a = a;
        this.b = b || (this.c = c);
    }
}, e2 = () => 100;

/*f*/
let f1 = {
    foo: 10,
    bar: "20"
}, f2 = null;"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: for _, varName := range []string{"a", "b", "c", "d", "e", "f"} {
}
