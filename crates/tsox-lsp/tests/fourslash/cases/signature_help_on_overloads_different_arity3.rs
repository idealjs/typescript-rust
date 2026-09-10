use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_on_overloads_different_arity3() {
    let content = r#"declare function f();
declare function f(s: string);
declare function f(s: string, b: boolean);
declare function f(n: number, b: boolean);

f(/**/"#;
    let mut s = Session::new_for_test("signatureHelpOnOverloadsDifferentArity3", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(): any", ParameterCount: 0, O
    fourslash::insert(&mut s, "x, ");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(s: string, b: boolean): any",
    fourslash::insert(&mut s, "x, ");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(s: string, b: boolean): any",
}
