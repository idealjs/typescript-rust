use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_spread_operator01() {
    let content = r#"let v = [1,2,3,4];
let x = [.../**/"#;
    let mut s = Session::new_for_test("completionListAfterSpreadOperator01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["v"], &[]);
}
