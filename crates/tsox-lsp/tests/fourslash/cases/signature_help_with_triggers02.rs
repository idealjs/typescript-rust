use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_with_triggers02() {
    let content = r#"declare function foo<T>(x: T, y: T): T;
declare function bar<U>(x: U, y: U): U;

foo(bar/*1*/)"#;
    let mut s = Session::new_for_test("signatureHelpWithTriggers02", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(x: unknown, y: unknown): un
    // TODO: f.Backspace(t, 1)
    fourslash::insert(&mut s, "<");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar<U>(x: U, y: U): U"})
    // TODO: f.Backspace(t, 1)
    fourslash::insert(&mut s, ",");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(x: <U>(x: U, y: U) => U, y:
    // TODO: f.Backspace(t, 1)
}
