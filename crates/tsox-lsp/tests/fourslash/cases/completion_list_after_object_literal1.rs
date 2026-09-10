use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_object_literal1() {
    let content = r#"var v = { x: 4, y: 3 }./**/"#;
    let mut s = Session::new_for_test("completionListAfterObjectLiteral1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x", "y"]);
}
