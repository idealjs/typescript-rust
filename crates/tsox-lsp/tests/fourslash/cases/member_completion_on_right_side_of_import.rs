use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_completion_on_right_side_of_import() {
    let content = r#"import x = M./**/"#;
    let mut s = Session::new_for_test("memberCompletionOnRightSideOfImport", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
