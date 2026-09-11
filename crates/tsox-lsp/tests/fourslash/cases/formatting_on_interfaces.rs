use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_interfaces() {
    let content = r#"/*1*/interface Blah 
{
}"#;
    let mut s = Session::new_for_test("formattingOnInterfaces", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"interface Blah {"#);
    // TODO: }
}
