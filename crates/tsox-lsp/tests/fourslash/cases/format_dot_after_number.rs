use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_dot_after_number() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"1+ 2 .toString() +3/*1*/
1+ 2. .toString() +3/*2*/
1+ 2.0 .toString() +3/*3*/
1+ (2) .toString() +3/*4*/
1+ 2_000 .toString() +3/*5*/"#;
    let mut s = Session::new_for_test("formatDotAfterNumber", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"1 + 2 .toString() + 3"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"1 + 2..toString() + 3"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"1 + 2.0.toString() + 3"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"1 + (2).toString() + 3"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"1 + 2_000 .toString() + 3"#);
}
