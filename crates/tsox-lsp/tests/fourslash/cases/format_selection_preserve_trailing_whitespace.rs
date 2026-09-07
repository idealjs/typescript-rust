use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts154 := f.GetOptions()"]
#[test]
fn format_selection_preserve_trailing_whitespace() {
    let content = r#"
/*begin*/;    
    
/*end*/    
    
"#;
    let mut s = Session::new(content);
    // TODO: opts154 := f.GetOptions()
    // TODO: opts154.FormatCodeSettings.TrimTrailingWhitespace = core.TSFalse
    fourslash::unsupported("Configure"); // f.Configure(t, opts154)
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
;    
    
    
    
"#,
    );
}
