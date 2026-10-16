use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_spread_expression02() {
    let content = r#"var x;
var y = (p) => [1,2,.../*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedSpreadExpression02", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["p", "x"], &[]);
}
