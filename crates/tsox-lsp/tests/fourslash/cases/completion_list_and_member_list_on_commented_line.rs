use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_and_member_list_on_commented_line() {
    let content = r#"// /**/
var"#;
    let mut s = Session::new_for_test("completionListAndMemberListOnCommentedLine", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
