use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Insert"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(a: Foo<I>, b: number): B"})
    fourslash::insert(&mut s, "null,");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(a: Foo<I>, b: number): B"})
    fourslash::unsupported("Insert"); // f.Insert(t, "10);")
}
