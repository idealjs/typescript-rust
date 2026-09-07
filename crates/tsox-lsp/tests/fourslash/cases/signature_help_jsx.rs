use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn signature_help_jsx() {
    let content = r#"//@Filename: test.tsx
//@jsx: react
declare var React: any;
const z = <div>{[].map(x => </**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyNoSignatureHelpWithContext"); // f.VerifyNoSignatureHelpWithContext(t, &lsproto.SignatureHelpContext{TriggerKind: lsproto.SignatureHe
    // TODO: }
}
