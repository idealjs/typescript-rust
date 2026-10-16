use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_for_optional_methods() {
    let content = r#"// @strict: true
interface Obj {
    optionalMethod?: (current: any) => any;
};

const o: Obj = {
  optionalMethod(/*1*/) {
    return {};
  }
};"#;
    let mut s = Session::new_for_test("signatureHelpForOptionalMethods", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "optionalMethod(current: any): a
}
