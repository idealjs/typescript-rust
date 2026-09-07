use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_typed_generic_prototype_member() {
    let content = r#"class C<T> {
   foo(x: T) { }
}
var /*1*/x = new C<any>(); // Quick Info for x is C<any>
var /*2*/y = C.prototype; // Quick Info for y is C<{}>"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var x: C<any>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var y: C<any>", "")
}
