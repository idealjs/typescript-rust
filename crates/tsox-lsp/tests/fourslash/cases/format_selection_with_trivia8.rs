use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_selection_with_trivia8() {
    let content = r#"/*begin*/;
    
/*end*/console.log();"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia8", content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, r#";

console.log();"#);
}
