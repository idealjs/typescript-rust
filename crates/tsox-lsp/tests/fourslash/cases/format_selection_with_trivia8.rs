use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_with_trivia8() {
    let content = r#"/*begin*/;
    
/*end*/console.log();"#;
    let mut s = Session::new_for_test("formatSelectionWithTrivia8", content);
    fourslash::format_selection(&mut s, "begin", "end");
    fourslash::verify_current_file_content(&mut s, r#";

console.log();"#);
}
