use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_generic_prototype_member() {
    let content = r#"class C<T> {
   foo(x: T) { }
}
var x = new /*1*/C<any>();
var y = C.proto/*2*/type;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "constructor C<any>(): C<any>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) C<T>.prototype: C<any>", "")
}
