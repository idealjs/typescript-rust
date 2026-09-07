use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn no_type_parameter_in_lhs() {
    let content = r#"interface I<T> { }
class C<T> {}
var /*1*/i: I<any>;
var /*2*/c: C<I>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var i: I<any>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var c: C<any>", "")
}
