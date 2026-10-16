use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_check_after_adding_generic_parameter() {
    let content = r#"function f<x, x>() { }
function f2<X, X>(b: X): X { return null; }
class C<X> {
    public f<x, x>() {}
f2<X>(b): X { return null; }
}

interface I<X, X> {
    f<X/*addTypeParam*/>();
    f2<X>(/*addParam*/a: X): X;
}
"#;
    let mut s = Session::new_for_test("typeCheckAfterAddingGenericParameter", content);
    fourslash::go_to_marker(&mut s, "addParam");
    fourslash::insert(&mut s, ", X");
    fourslash::go_to_marker(&mut s, "addTypeParam");
    fourslash::insert(&mut s, ", X");
}
