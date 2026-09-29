use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_slash() {
    let content = r#"var a = 0;
a/./**/"#;
    let mut s = Session::new_for_test("completionListAfterSlash", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
