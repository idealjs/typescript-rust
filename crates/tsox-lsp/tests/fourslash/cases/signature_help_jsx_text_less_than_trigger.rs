use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_jsx_text_less_than_trigger() {
    let content = r#"//@Filename: test.tsx
//@jsx: react
declare var React: any;
declare function Text(props: { children?: any }): any;

const text = () => {
	return <Text>/*m*/</Text>;
};"#;
    let mut s = Session::new_for_test("signatureHelpJsxTextLessThanTrigger", content);
    fourslash::go_to_marker(&mut s, "m");
    fourslash::insert(&mut s, "<");
    // TODO: f.VerifyNoSignatureHelpWithContext(t, &lsproto.SignatureHelpContext{
}
