use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_parameter() {
    let content = r#"function foo(
    first:
    number,/*first*/
    second: (
    string/*second*/
    ),
    third:
    (
    boolean/*third*/
    )
) {
}"#;
    let mut s = Session::new_for_test("formatParameter", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "first");
    fourslash::verify_current_line_content(&mut s, r#"        number,"#);
    fourslash::go_to_marker(&mut s, "second");
    fourslash::verify_current_line_content(&mut s, r#"        string"#);
    fourslash::go_to_marker(&mut s, "third");
    fourslash::verify_current_line_content(&mut s, r#"            boolean"#);
}
