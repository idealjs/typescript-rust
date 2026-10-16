use tsox_lsp::fourslash::{self, Session};


#[test]
fn rest_arg_signature_help() {
    let content = r#"function f(...x: any[]) { }
f(/**/);"#;
    let mut s = Session::new_for_test("restArgSignatureHelp", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "x", IsVariadic: true, 
}
