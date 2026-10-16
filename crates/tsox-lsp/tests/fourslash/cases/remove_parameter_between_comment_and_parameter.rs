use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_parameter_between_comment_and_parameter() {
    let content = r#"function fn(/* comment! */ /**/a: number, c) { }"#;
    let mut s = Session::new_for_test("removeParameterBetweenCommentAndParameter", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 10)
}
