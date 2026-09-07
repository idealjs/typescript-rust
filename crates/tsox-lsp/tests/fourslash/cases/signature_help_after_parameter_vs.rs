use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_after_parameter_vs() {
    let content = r#"type Type = (a, b, c) => void
const a: Type = (a/*1*/, b/*2*/) => {}
const b: Type = function (a/*3*/, b/*4*/) {}
const c: Type = ({ /*5*/a: { b/*6*/ }}/*7*/ = { }/*8*/, [b/*9*/]/*10*/, .../*11*/c/*12*/) => {}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
