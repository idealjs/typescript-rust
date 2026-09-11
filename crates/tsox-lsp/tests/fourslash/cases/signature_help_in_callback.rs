use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_in_callback() {
    let content = r#"declare function forEach(f: () => void);
forEach(/*1*/() => {
    /*2*/
});"#;
    let mut s = Session::new_for_test("signatureHelpInCallback", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "forEach(f: () => void): any"})
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "2")
}
