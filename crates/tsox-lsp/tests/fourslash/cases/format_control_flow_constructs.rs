use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_control_flow_constructs() {
    let content = r#"if (true)/**/
{     
}"#;
    let mut s = Session::new_for_test("formatControlFlowConstructs", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"if (true) {"#);
    // TODO: }
}
