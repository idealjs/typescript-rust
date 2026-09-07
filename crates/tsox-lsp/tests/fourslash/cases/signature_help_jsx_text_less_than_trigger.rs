use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpWithContext"]
#[test]
fn signature_help_jsx_text_less_than_trigger() {
    let content = r#"//@Filename: test.tsx
//@jsx: react
declare var React: any;
declare function Text(props: { children?: any }): any;

const text = () => {
	return <Text>/*m*/</Text>;
};"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "m");
    fourslash::insert(&mut s, "<");
    fourslash::unsupported("VerifyNoSignatureHelpWithContext"); // f.VerifyNoSignatureHelpWithContext(t, &lsproto.SignatureHelpContext{
}
