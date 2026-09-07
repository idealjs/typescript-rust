use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_typed_generic_prototype_member() {
    let content = r#"class C<T> {
   foo(x: T) { }
}
var /*1*/x = new C<any>(); // Quick Info for x is C<any>
var /*2*/y = C.prototype; // Quick Info for y is C<{}>"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var x: C<any>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var y: C<any>", "");
}
