use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn generic_method_param() {
    let content = r#"class C<T> {
    /*1*/
}
/*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "constructor(){}")
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "foo(a: T) {")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "var x = new C<number>();")
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "var y: number = x.foo(5);")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
