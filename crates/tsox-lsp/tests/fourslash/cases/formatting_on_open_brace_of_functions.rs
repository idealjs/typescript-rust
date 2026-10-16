use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_open_brace_of_functions() {
    let content = r#"/**/function T2_y()
{
Plugin.T1.t1_x();
}"#;
    let mut s = Session::new_for_test("formattingOnOpenBraceOfFunctions", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"function T2_y() {"#);
    // TODO: }
}
