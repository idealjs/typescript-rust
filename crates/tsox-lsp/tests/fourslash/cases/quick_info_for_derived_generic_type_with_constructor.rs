use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_derived_generic_type_with_constructor() {
    let content = r#"class A<T> {
    foo() { }
}
class B<T> extends A<T> {
    bar() { }
    constructor() { super() }
}
class B2<T> extends A<T> {
    bar() { }
}
var /*1*/b: B<number>;
var /*2*/b2: B<number>;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var b: B<number>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var b2: B<number>", "");
}
