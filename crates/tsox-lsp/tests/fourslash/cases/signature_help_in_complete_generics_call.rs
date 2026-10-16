use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_in_complete_generics_call() {
    let content = r#"function foo<T>(x: number, callback: (x: T) => number) {
}
foo(/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpInCompleteGenericsCall", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(x: number, callback: (x: un
}
