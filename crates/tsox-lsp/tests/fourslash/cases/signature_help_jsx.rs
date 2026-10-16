use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_jsx() {
    let content = r#"//@Filename: test.tsx
//@jsx: react
declare var React: any;
const z = <div>{[].map(x => </**/"#;
    let mut s = Session::new_for_test("signatureHelpJSX", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyNoSignatureHelpWithContext(t, &lsproto.SignatureHelpContext{TriggerKind: lsproto.SignatureHe
    // TODO: }
}
