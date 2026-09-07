use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn hover_over_private_name() {
    let content = r#"class A {
    #f/*1*/oo = 3;
    #b/*2*/ar: number;
    #b/*3*/az = () => "hello";
    #q/*4*/ux(n: number): string {
        return "" + n;
    }
    static #staticF/*5*/oo = 3;
    static #staticB/*6*/ar: number;
    static #staticB/*7*/az = () => "hello";
    static #staticQ/*8*/ux(n: number): string {
        return "" + n;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) A.#foo: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) A.#bar: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(property) A.#baz: () => string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(method) A.#qux(n: number): string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(property) A.#staticFoo: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(property) A.#staticBar: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(property) A.#staticBaz: () => string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8", "(method) A.#staticQux(n: number): string", "")
}
