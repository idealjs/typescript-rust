use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_type_of_expression02() {
    let content = r#"var x;
var y = (p) => typeof /*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedTypeOfExpression02", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["x", "p"], &[]);
}
