use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_else_inside_a_function() {
    let content = r#"var x = function() {
    if (true) {
    /*1*/} else {/*2*/
}

// newline at the end of the file"#;
    let mut s = Session::new_for_test("formattingElseInsideAFunction", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert_line(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    } else {"#);
    // TODO: }
}
