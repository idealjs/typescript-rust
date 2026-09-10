use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_on_nested_overloads() {
    let content = r#"declare function fn(x: string);
declare function fn(x: string, y: number);
declare function fn2(x: string);
declare function fn2(x: string, y: number);
fn('', fn2(/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpOnNestedOverloads", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fn2(x: string): any", Parameter
    fourslash::insert(&mut s, "'',");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fn2(x: string, y: number): any"
}
