use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_preserve_trailing_whitespace() {
    let content = r#"
/*begin*/;    
    
/*end*/    
    
"#;
    let mut s = Session::new_for_test("formatSelectionPreserveTrailingWhitespace", content);
    // TODO: opts154 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("trim_trailing_whitespace", "false")]);
    fourslash::format_selection(&mut s, "begin", "end");
    fourslash::verify_current_file_content(&mut s, r#"
;    
    
    
    
"#);
}
