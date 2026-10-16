use tsox_lsp::fourslash::{self, Session};


#[test]
fn as_operator_completion() {
    let content = r#"type T = number;
var x;
var y = x as /**/"#;
    let mut s = Session::new_for_test("asOperatorCompletion", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["T"], &[]);
}
