use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn generic_function_signature_help2() {
    let content = r#"var f = <T>(a: T) => a;
f(/**/"#;
    let mut s = Session::new_for_test("genericFunctionSignatureHelp2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: unknown): unknown"})
}
