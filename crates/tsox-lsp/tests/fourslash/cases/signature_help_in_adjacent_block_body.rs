use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelpPresent"]
#[test]
fn signature_help_in_adjacent_block_body() {
    let content = r#"declare function foo(...args);

foo(() => {/*1*/}/*2*/)"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelpPresent"); // f.VerifySignatureHelpPresent(t, &lsproto.SignatureHelpContext{TriggerKind: lsproto.SignatureHelpTrig
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelpPresent"); // f.VerifySignatureHelpPresent(t, &lsproto.SignatureHelpContext{TriggerKind: lsproto.SignatureHelpTrig
}
