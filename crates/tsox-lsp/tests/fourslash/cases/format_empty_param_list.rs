use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_empty_param_list() {
    let content = r#"function f( f: function){/*1*/"#;
    let mut s = Session::new_for_test("formatEmptyParamList", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "}");
    fourslash::verify_current_line_content(&mut s, r#"function f(f: function) { }"#);
}
