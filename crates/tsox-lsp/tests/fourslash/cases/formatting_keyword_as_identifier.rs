use tsox_lsp::fourslash::{self, Session};

#[test]
fn formatting_keyword_as_identifier() {
    let content = r#"declare var module/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"declare var module;"#);
}
