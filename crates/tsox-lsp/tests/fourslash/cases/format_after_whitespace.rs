use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn format_after_whitespace() {
    let content = r#"function foo()
{
    var bar;
    /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"function foo()
{
    var bar;


}"#,
    );
}
