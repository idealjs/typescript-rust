use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_empty_file() {
    let content = r#"var a = 0;
/**/"#;
    let mut s = Session::new_for_test("completionListInEmptyFile", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["a"], &[]);
}
