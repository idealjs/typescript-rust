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
    fourslash::insert_line(&mut s, "constructor(){}");
    fourslash::insert_line(&mut s, "foo(a: T) {");
    fourslash::insert_line(&mut s, "    return a;");
    fourslash::insert_line(&mut s, "}");
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert_line(&mut s, "var x = new C<number>();");
    fourslash::insert_line(&mut s, "var y: number = x.foo(5);");
    fourslash::verify_no_errors(&mut s, );
}
