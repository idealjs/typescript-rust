use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_type_parameters_not_variadic() {
    let content = r#"declare function f(a: any, ...b: any[]): any;
f</*1*/>(1, 2);"#;
    let mut s = Session::new_for_test("signatureHelpTypeParametersNotVariadic", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{IsVariadic: false, IsVariadicSet: true
}
