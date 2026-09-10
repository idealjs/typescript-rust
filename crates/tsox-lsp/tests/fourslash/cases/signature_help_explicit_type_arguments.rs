use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_explicit_type_arguments() {
    let content = r#"declare function f<T = boolean, U = string>(x: T, y: U): T;
f<number, string>(/*1*/);
f(/*2*/);
f<number>(/*3*/);
f<number, string, boolean>(/*4*/);
interface A { a: number }
interface B extends A { b: string }
declare function g<T, U, V extends A = B>(x: T, y: U, z: V): T;
declare function h<T, U, V extends A>(x: T, y: U, z: V): T;
declare function j<T, U, V = B>(x: T, y: U, z: V): T;
g(/*5*/);
h(/*6*/);
j(/*7*/);
g<number>(/*8*/);
h<number>(/*9*/);
j<number>(/*10*/);"#;
    let mut s = Session::new_for_test("signatureHelpExplicitTypeArguments", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: boolean, y: string): boole
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "g(x: unknown, y: unknown, z: B)
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "h(x: unknown, y: unknown, z: A)
    fourslash::go_to_marker(&mut s, "7");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "j(x: unknown, y: unknown, z: B)
    fourslash::go_to_marker(&mut s, "8");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "g(x: number, y: unknown, z: B):
    fourslash::go_to_marker(&mut s, "9");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "h(x: number, y: unknown, z: A):
    fourslash::go_to_marker(&mut s, "10");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "j(x: number, y: unknown, z: B):
}
