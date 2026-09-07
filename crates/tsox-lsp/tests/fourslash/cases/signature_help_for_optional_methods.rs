use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "optionalMethod(current: any): a
}
