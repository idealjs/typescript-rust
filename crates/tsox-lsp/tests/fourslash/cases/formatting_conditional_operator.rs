use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_conditional_operator() {
    let content = r#"var x=true?1:2"#;
    let mut s = Session::new_for_test("formattingConditionalOperator", content);
    // TODO: f.FormatDocument(t, "")
    // TODO: f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"var x = true ? 1 : 2"#);
}
