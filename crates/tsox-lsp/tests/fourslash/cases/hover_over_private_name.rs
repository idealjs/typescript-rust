use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "(property) A.#foo: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) A.#bar: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(property) A.#baz: () => string", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(method) A.#qux(n: number): string", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(property) A.#staticFoo: number", "");
    fourslash::verify_quick_info_at(&mut s, "6", "(property) A.#staticBar: number", "");
    fourslash::verify_quick_info_at(&mut s, "7", "(property) A.#staticBaz: () => string", "");
    fourslash::verify_quick_info_at(&mut s, "8", "(method) A.#staticQux(n: number): string", "");
}
