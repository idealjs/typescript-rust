use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn extend_interface_overloaded_method() {
    let content = r#"// @strict: false
interface A<T> {
    foo(a: T): B<T>;
    foo(): void ;
    foo2(): B<number>;
}
interface B<T> extends A<T> {
    bar(): void ;
}
var b: B<number>;
var /**/x = b.foo2().foo(5).foo(); // 'x' is of type 'void'"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var x: void", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
