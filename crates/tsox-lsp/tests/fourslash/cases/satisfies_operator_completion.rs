use tsox_lsp::fourslash::{self, Session};


#[test]
fn satisfies_operator_completion() {
    let content = r#"type T = number;
var x;
var y = x satisfies /**/"#;
    let mut s = Session::new_for_test("satisfiesOperatorCompletion", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["T"], &[]);
}
