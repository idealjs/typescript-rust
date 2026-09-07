use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn overload_on_const_call_signature() {
    let content = r#"var foo: {
    (name: string): string;
    (name: 'order'): string;
    (name: 'content'): string;
    (name: 'done'): string;
}
var /*2*/x = foo(/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(name: 'order'): string", Ov
    fourslash::insert(&mut s, "\"hi\"");
    fourslash::verify_quick_info_at(&mut s, "2", "var x: string", "");
}
