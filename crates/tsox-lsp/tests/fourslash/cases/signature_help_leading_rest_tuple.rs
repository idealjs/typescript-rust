use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_leading_rest_tuple() {
    let content = r#"export function leading(...args: [...names: string[], allCaps: boolean]): void {
}

leading(/*1*/);
leading("ok", /*2*/);
leading("ok", "ok", /*3*/);"#;
    let mut s = Session::new_for_test("signatureHelpLeadingRestTuple", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(...names: string[], all
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(...names: string[], all
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "leading(...names: string[], all
}
