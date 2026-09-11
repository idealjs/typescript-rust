use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_after_object_literal() {
    let content = r#"/**/namespace Default{var x= ( { } ) ;}"#;
    let mut s = Session::new_for_test("formatAfterObjectLiteral", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"namespace Default { var x = ({}); }"#);
}
