use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_trailing_rest_tuple() {
    let content = r#"export function leading(allCaps: boolean, ...names: string[]): void {
}

leading(/*1*/);
leading(false, /*2*/);
leading(false, "ok", /*3*/);"#;
    let mut s = Session::new_for_test("signatureHelpTrailingRestTuple", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(allCaps: boolean, ...na
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(allCaps: boolean, ...na
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(allCaps: boolean, ...na
}
