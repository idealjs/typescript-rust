use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_with_multiline_comments() {
    let content = r#"f(/*
/*2*/         */() => { /*1*/ });"#;
    let mut s = Session::new_for_test("formattingWithMultilineComments", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert_line(&mut s, "");
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"         */() => {"#);
    // TODO: }
}
