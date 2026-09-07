use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_generic_prototype_member() {
    let content = r#"class C<T> {
   foo(x: T) { }
}
var x = new /*1*/C<any>();
var y = C.proto/*2*/type;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "constructor C<any>(): C<any>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) C<T>.prototype: C<any>", "");
}
