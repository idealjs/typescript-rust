use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_in_destructuring4() {
    let content = r#"/*1*/const { 
/*2*/    a,
/*3*/    b,
/*4*/} = { a: 1, b: 2 };"#;
    let mut s = Session::new_for_test("formattingInDestructuring4", content);
    // TODO: opts198 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_opening_and_before_closing_nonempty_braces", "false")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"const {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    a,"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    b,"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"} = {a: 1, b: 2};"#);
}
