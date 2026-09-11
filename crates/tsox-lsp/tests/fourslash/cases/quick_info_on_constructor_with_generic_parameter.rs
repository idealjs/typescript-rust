use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_constructor_with_generic_parameter() {
    let content = r#"interface I {
    x: number;
}
class Foo<T> {
    y: T;
}
class A {
    foo() { }
}
class B extends A {
    constructor(a: Foo<I>, b: number) {
        super();
    }
}
var x = new /*2*/B(/*1*/"#;
    let mut s = Session::new_for_test("quickInfoOnConstructorWithGenericParameter", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(a: Foo<I>, b: number): B"})
    fourslash::insert(&mut s, "null,");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(a: Foo<I>, b: number): B"})
    // TODO: f.Insert(t, "10);")
}
