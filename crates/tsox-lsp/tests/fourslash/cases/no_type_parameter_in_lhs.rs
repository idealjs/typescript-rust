use tsox_lsp::fourslash::{self, Session};


#[test]
fn no_type_parameter_in_lhs() {
    let content = r#"interface I<T> { }
class C<T> {}
var /*1*/i: I<any>;
var /*2*/c: C<I>;"#;
    let mut s = Session::new_for_test("noTypeParameterInLHS", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var i: I<any>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var c: C<any>", "");
}
