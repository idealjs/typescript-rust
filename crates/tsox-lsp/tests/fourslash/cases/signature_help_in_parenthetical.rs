use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_in_parenthetical() {
    let content = r#"class base { constructor (public n: number, public y: string) { } }
(new base(/**/"#;
    let mut s = Session::new_for_test("signatureHelpInParenthetical", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "n"})
    fourslash::insert(&mut s, "0, ");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "y"})
}
