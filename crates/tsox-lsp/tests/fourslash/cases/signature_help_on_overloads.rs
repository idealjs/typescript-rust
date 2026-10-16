use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_on_overloads() {
    let content = r#"declare function fn(x: string);
declare function fn(x: string, y: number);
fn(/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpOnOverloads", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fn(x: string): any", ParameterN
    fourslash::insert(&mut s, "'',");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fn(x: string, y: number): any",
}
