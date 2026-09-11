use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_comments2() {
    let content = r#"// */{| "name" : "1" |}"#;
    let mut s = Session::new_for_test("completionListInComments2", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
}
