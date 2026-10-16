use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_call_expression() {
    let content = r#"let x = someCall() /**/"#;
    let mut s = Session::new_for_test("completionAfterCallExpression", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["satisfies", "as"], &[]);
}
