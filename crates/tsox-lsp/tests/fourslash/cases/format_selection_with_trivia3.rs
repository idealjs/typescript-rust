use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_with_trivia3() {
    let content = r#"if (true) {
/*begin*/// test comment
/*end*/}"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia3", content);
    // TODO: f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, r#"if (true) {
    // test comment
}"#);
}
