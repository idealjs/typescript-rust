use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_with_trivia6() {
    let content = r#"/*begin*/    // test comment
/*end*/"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia6", content);
    // TODO: f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, r#"// test comment
"#);
}
