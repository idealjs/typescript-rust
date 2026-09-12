use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_crash() {
    let content = r#"/**/module Default{ 
}"#;
    let mut s = Session::new_for_test("formattingCrash", content);
    // TODO: opts131 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_functions", "true")]);
    // TODO: opts199 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"module Default"#);
}
