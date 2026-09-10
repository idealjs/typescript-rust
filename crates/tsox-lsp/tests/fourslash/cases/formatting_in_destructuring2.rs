use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn formatting_in_destructuring2() {
    let content = r#"/*1*/function   drawText(    { text = "", location: [x, y]=           [0, 0], bold = false }) {
    // Draw text  
}"#;
    let mut s = Session::new_for_test("formattingInDestructuring2", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function drawText({ text = "", location: [x, y] = [0, 0], bold = false }) {"#);
    // TODO: }
}
