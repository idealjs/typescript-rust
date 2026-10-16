use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_signature_help() {
    let content = r#"interface C {
   (): number;
}
var c: C;
c(/**/"#;
    let mut s = Session::new_for_test("callSignatureHelp", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "c(): number"})
}
