use tsox_lsp::fourslash::{self, Session};

#[test]
fn formatting_expressions_in_if_condition() {
    let content = r#"if (a === 1 ||
    /*0*/b === 2 ||/*1*/
    c === 3) {
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_marker(&mut s, "0");
    fourslash::verify_current_line_content(&mut s, r#"    b === 2 ||"#);
}
