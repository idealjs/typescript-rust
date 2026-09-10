use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_conditional_operator() {
    let content = r#"var x=true?1:2"#;
    let mut s = Session::new_for_test("formattingConditionalOperator", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"var x = true ? 1 : 2"#);
}
