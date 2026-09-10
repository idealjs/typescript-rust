use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_comments() {
    let content = r#"var foo = '';
( // f/**/"#;
    let mut s = Session::new_for_test("completionListInComments", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
