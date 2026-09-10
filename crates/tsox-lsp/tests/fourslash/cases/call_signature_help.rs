use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn call_signature_help() {
    let content = r#"interface C {
   (): number;
}
var c: C;
c(/**/"#;
    let mut s = Session::new_for_test("callSignatureHelp", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "c(): number"})
}
