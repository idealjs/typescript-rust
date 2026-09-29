use tsox_lsp::fourslash::{self, Session};


#[test]
fn error_in_incomplete_method_in_object_literal() {
    let content = r#"var x: { f(): string } = { f( }"#;
    let mut s = Session::new_for_test("errorInIncompleteMethodInObjectLiteral", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
