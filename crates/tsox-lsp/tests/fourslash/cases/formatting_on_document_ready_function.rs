use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_document_ready_function() {
    let content = r#"/*1*/$    (   document   )   .  ready  (   function   (   )   {
/*2*/    alert    (           'i am ready'  )   ;
/*3*/           }                 );"#;
    let mut s = Session::new_for_test("formattingOnDocumentReadyFunction", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"$(document).ready(function() {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    alert('i am ready');"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"});"#);
}
