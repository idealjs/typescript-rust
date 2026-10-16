use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_element_access_expression02() {
    let content = r#"var x;
var y = (p) => x[/*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedElementAccessExpression02", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["p", "x"], &[]);
}
