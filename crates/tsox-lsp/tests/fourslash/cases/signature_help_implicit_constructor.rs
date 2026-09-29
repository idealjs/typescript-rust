use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_implicit_constructor() {
    let content = r#"class ImplicitConstructor {
}
var implicitConstructor = new ImplicitConstructor(/**/);"#;
    let mut s = Session::new_for_test("signatureHelpImplicitConstructor", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "ImplicitConstructor(): Implicit
}
