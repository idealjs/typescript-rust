use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_templates_with_newline() {
    let content = r#"` + "`" + `${1}` + "`" + `;
` + "`" + "#;
    // TODO: ` + "`" + `;/**/1`
    let mut s = Session::new_for_test("formattingTemplatesWithNewline", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "\n");
    fourslash::verify_current_line_content(&mut s, r#"1"#);
}
