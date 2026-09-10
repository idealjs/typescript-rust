use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_expanded_tuples_argument_index() {
    let content = r#"function foo(...args: [string, string] | [number, string, string]
) {

}

foo(123/*1*/,)
foo(""/*2*/, ""/*3*/)
foo(123/*4*/, ""/*5*/, )
foo(123/*6*/, ""/*7*/, ""/*8*/)"#;
    let mut s = Session::new_for_test("signatureHelpExpandedTuplesArgumentIndex", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: string, args_1: str
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: string, args_1: str
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
    fourslash::go_to_marker(&mut s, "7");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
    fourslash::go_to_marker(&mut s, "8");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(args_0: number, args_1: str
}
