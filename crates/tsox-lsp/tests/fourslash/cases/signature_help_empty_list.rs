use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_empty_list() {
    let content = r#"function Foo(arg1: string, arg2: string) {
}

Foo(/*1*/);
function Bar<T>(arg1: string, arg2: string) { }
Bar</*2*/>();"#;
    let mut s = Session::new_for_test("signatureHelpEmptyList", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Foo(arg1: string, arg2: string)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Bar<T>(arg1: string, arg2: stri
}
