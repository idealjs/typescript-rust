use tsox_lsp::fourslash::{self, Session};


#[test]
fn edit_lambda_arg_to_type_parameter1() {
    let content = r#"class C<T> {
    foo(x: T) {
        return (a: number/*1*/) => x;
    }
}
/*2*/"#;
    let mut s = Session::new_for_test("editLambdaArgToTypeParameter1", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.Backspace(t, 6)
    fourslash::insert(&mut s, "T");
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert_line(&mut s, "");
    fourslash::verify_no_errors(&mut s, );
}
