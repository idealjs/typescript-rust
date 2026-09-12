use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_single_line_with_new_line_option_set() {
    let content = r#"/*1*/namespace Default{}
/*2*/function foo(){}
/*3*/if (true){}
/*4*/function boo() {
}"#;
    let mut s = Session::new_for_test("formattingSingleLineWithNewLineOptionSet", content);
    // TODO: opts211 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_functions", "true")]);
    // TODO: opts279 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"namespace Default { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"function foo() { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"if (true) { }"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"function boo()"#);
}
