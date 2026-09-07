use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn generic_function_return_type() {
    let content = r#"function foo<T, U>(x: T, y: U): (a: U) => T {
    var z = y;
    return (z) => x;
}
var /*2*/r = foo(/*1*/1, "");
var /*4*/r2 = r(/*3*/"");"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(x: number, y: string): (a: 
    fourslash::verify_quick_info_at(&mut s, "2", "var r: (a: string) => number", "");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "r(a: string): number"})
    fourslash::verify_quick_info_at(&mut s, "4", "var r2: number", "");
}
