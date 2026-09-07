use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn multiline_comment_before_open_brace() {
    let content = r#"function test() /*1*//* %^ */
{
    if (true) /*2*//* %^ */
    {
    }
}
function a() {
    /* %^ */ }/*3*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function test() /* %^ */ {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    if (true) /* %^ */ {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    // TODO: }
}
