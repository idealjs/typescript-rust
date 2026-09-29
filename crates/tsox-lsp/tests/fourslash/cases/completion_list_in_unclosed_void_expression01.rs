use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_void_expression01() {
    let content = r#"var x;
var y = (p) => void /*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedVoidExpression01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["p", "x"], &[]);
}
