use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_function_signature_help1() {
    let content = r#"function f<T>(a: T): T { return null; }
f(/**/"#;
    let mut s = Session::new_for_test("genericFunctionSignatureHelp1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: unknown): unknown"})
}
