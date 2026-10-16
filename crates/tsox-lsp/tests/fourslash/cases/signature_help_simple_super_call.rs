use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_simple_super_call() {
    let content = r#"class SuperCallBase {
    constructor(b: boolean) {
    }
}
class SuperCall extends SuperCallBase {
    constructor() {
        super(/*superCall*/);
    }
}"#;
    let mut s = Session::new_for_test("signatureHelpSimpleSuperCall", content);
    fourslash::go_to_marker(&mut s, "superCall");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "SuperCallBase(b: boolean): Supe
}
