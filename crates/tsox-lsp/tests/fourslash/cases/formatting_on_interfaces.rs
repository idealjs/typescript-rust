use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn formatting_on_interfaces() {
    let content = r#"/*1*/interface Blah 
{
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"interface Blah {"#);
    // TODO: }
}
