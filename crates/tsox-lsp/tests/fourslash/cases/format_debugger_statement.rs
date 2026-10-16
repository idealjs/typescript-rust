use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_debugger_statement() {
    let content = r#"if(false){debugger;}
  if    (   false   )   {    debugger  ;   }"#;
    let mut s = Session::new_for_test("formatDebuggerStatement", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_bof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"if (false) { debugger; }"#);
    fourslash::go_to_eof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"if (false) { debugger; }"#);
}
