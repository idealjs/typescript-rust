use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn edit_lambda_arg_to_type_parameter1() {
    let content = r#"class C<T> {
    foo(x: T) {
        return (a: number/*1*/) => x;
    }
}
/*2*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Backspace"); // f.Backspace(t, 6)
    fourslash::insert(&mut s, "T");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
