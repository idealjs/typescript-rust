use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_regular_expression_literal05() {
    let content = r#"let v = 100;
let x = /absidey/g/**/"#;
    let mut s = Session::new_for_test("completionListAfterRegularExpressionLiteral05", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
