use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_method_param() {
    let content = r#"class C<T> {
    /*1*/
}
/*2*/"#;
    let mut s = Session::new_for_test("genericMethodParam", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.InsertLine(t, "constructor(){}")
    // TODO: f.InsertLine(t, "foo(a: T) {")
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.InsertLine(t, "var x = new C<number>();")
    // TODO: f.InsertLine(t, "var y: number = x.foo(5);")
    fourslash::verify_no_errors(&mut s, );
}
