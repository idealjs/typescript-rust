use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_doc_comment_in_block() {
    let content = r#"{
    /*1*//**
     * Some doc comment
     *//*2*/
    const a = 1;
}

while (true) {
/*3*//**
 * Some doc comment
 *//*4*/
}"#;
    let mut s = Session::new_for_test("formatSelectionDocCommentInBlock", content);
    // TODO: f.FormatSelection(t, "1", "2")
    fourslash::verify_current_file_content(&mut s, r#"{
    /**
     * Some doc comment
     */
    const a = 1;
}

while (true) {
/**
 * Some doc comment
 */
}"#);
    // TODO: f.FormatSelection(t, "3", "4")
    fourslash::verify_current_file_content(&mut s, r#"{
    /**
     * Some doc comment
     */
    const a = 1;
}

while (true) {
    /**
     * Some doc comment
     */
}"#);
}
