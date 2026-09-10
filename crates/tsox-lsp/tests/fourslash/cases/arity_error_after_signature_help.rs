use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn arity_error_after_signature_help() {
    let content = r#"// @strict: true

declare function f(x: string, y: number): any;

/*1*/f/*2*/(/*3*/)"#;
    let mut s = Session::new_for_test("arityErrorAfterSignatureHelp", content);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{})
    fourslash::insert(&mut s, "\"");
    fourslash::insert(&mut s, "\"");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{})
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
}
