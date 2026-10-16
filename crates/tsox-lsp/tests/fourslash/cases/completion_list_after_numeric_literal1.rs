use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_numeric_literal1() {
    let content = r#"5../**/"#;
    let mut s = Session::new_for_test("completionListAfterNumericLiteral1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["toExponential", "toFixed", "toLocaleString", "toPrecision", "toString", "valueOf"]);
}
