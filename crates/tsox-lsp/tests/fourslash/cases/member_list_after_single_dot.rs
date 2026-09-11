use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_after_single_dot() {
    let content = r#"./**/"#;
    let mut s = Session::new_for_test("memberListAfterSingleDot", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
