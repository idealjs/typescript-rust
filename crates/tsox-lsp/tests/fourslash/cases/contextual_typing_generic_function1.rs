use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn contextual_typing_generic_function1() {
    let content = r#"var obj: { f<T>(x: T): T } = { f: <S>(/*1*/x) => x };
var obj2: <T>(x: T) => T = <S>(/*2*/x) => x;

class C<T> {
    obj: <T>(x: T) => T
}
var c = new C();
c.obj = <S>(/*3*/x) => x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) x: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) x: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) x: any", "")
}
