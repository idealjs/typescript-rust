use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_with_trivia4() {
    let content = r#"if (true) {
/*begin*/// test comment
/*end*/console.log();
}"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia4", content);
    // TODO: f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, r#"if (true) {
    // test comment
console.log();
}"#);
}
