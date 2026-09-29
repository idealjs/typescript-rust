use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_on_overload_on_const() {
    let content = r#"function x1(x: 'hi');
function x1(y: 'bye');
function x1(z: string);
function x1(a: any) {
}

x1(''/*1*/);
x1('hi'/*2*/);
x1('bye'/*3*/);"#;
    let mut s = Session::new_for_test("signatureHelpOnOverloadOnConst", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "z", ParameterSpan: "z:
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "x", ParameterSpan: "x:
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "y", ParameterSpan: "y:
}
