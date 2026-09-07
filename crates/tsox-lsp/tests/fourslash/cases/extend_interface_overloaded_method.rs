use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "", "var x: void", "");
    fourslash::verify_no_errors(&mut s);
}
