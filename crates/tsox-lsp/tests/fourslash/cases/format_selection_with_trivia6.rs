use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_selection_with_trivia6() {
    let content = r#"/*begin*/    // test comment
/*end*/"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia6", content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, r#"// test comment
"#);
}
