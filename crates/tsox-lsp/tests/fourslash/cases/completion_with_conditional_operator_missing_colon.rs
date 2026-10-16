use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_with_conditional_operator_missing_colon() {
    let content = r#"1 ? fun/*1*/
function func () {}"#;
    let mut s = Session::new_for_test("completionWithConditionalOperatorMissingColon", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["func"], &[]);
}
