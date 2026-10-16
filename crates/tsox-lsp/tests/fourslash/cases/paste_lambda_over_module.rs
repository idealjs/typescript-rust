use tsox_lsp::fourslash::{self, Session};


#[test]
fn paste_lambda_over_module() {
    let content = r#"// @strict: false
/**/"#;
    let mut s = Session::new_for_test("pasteLambdaOverModule", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::paste(&mut s, "namespace B { }");
    fourslash::go_to_bof(&mut s, );
    // TODO: f.DeleteAtCaret(t, 15)
    fourslash::insert(&mut s, "var t = (public x) => { };");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
