use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_enter_in_strings() {
    let content = r#"var x = /*1*/"unclosed string literal\/*2*/"#;
    let mut s = Session::new_for_test("formattingOnEnterInStrings", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert_line(&mut s, "");
    fourslash::insert_line(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, "var x = \"unclosed string literal\\");
}
