use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn format_trim_remaining_range() {
    let content = r#"// @lib: es5
    ;
    /*
    
*/"#;
    let mut s = Session::new_for_test("formatTrimRemainingRange", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#";
/*
 
*/"#);
}
