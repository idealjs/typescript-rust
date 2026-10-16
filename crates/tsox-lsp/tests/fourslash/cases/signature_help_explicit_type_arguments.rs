use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: boolean, y: string): boole
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(x: number, y: string): number
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "g(x: unknown, y: unknown, z: B)
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "h(x: unknown, y: unknown, z: A)
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "j(x: unknown, y: unknown, z: B)
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "g(x: number, y: unknown, z: B):
    fourslash::go_to_marker(&mut s, "9");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "h(x: number, y: unknown, z: A):
    fourslash::go_to_marker(&mut s, "10");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "j(x: number, y: unknown, z: B):
}
