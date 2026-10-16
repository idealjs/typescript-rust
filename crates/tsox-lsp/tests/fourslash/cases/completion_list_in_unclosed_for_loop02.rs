use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_for_loop02() {
    let content = r#"for (let i = 0; i < 10; i++) /*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedForLoop02", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["i"], &[]);
}
