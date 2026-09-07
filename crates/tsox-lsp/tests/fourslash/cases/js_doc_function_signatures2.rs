use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn js_doc_function_signatures2() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {(arg0: string, arg1?: boolean) => number} */
var f6;

f6('', /**/false)"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f6(arg0: string, arg1?: boolean
}
