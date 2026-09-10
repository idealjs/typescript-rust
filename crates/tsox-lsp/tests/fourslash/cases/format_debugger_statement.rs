use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_debugger_statement() {
    let content = r#"if(false){debugger;}
  if    (   false   )   {    debugger  ;   }"#;
    let mut s = Session::new_for_test("formatDebuggerStatement", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"if (false) { debugger; }"#);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::verify_current_line_content(&mut s, r#"if (false) { debugger; }"#);
}
