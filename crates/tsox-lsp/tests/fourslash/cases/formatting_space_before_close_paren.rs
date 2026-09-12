use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_space_before_close_paren() {
    let content = r#"/*1*/({});
/*2*/(  {});
/*3*/({foo:42});
/*4*/(  {foo:42}  );
/*5*/var bar = (function (a) { });"#;
    let mut s = Session::new_for_test("formattingSpaceBeforeCloseParen", content);
    // TODO: opts235 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_opening_and_before_closing_nonempty_parenthesis", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"( {} );"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"( {} );"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"( { foo: 42 } );"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"( { foo: 42 } );"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"var bar = ( function( a ) { } );"#);
    // TODO: opts674 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_opening_and_before_closing_nonempty_parenthesis", "false")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"({});"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"({});"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"({ foo: 42 });"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"({ foo: 42 });"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"var bar = (function(a) { });"#);
}
