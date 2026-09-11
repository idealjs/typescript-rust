use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_end_of_word_in_arrow_function01() {
    let content = r#"xyz => x/*1*/"#;
    let mut s = Session::new_for_test("completionListAtEndOfWordInArrowFunction01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["xyz"], &[]);
}
