use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_on_overloads_different_arity() {
    let content = r#"declare function f(s: string);
declare function f(n: number);
declare function f(s: string, b: boolean);
declare function f(n: number, b: boolean);

f(1/**/"#;
    let mut s = Session::new_for_test("signatureHelpOnOverloadsDifferentArity", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(n: number): any", ParameterNa
    fourslash::insert(&mut s, ", ");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(n: number, b: boolean): any",
}
