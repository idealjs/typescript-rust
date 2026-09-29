use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_space_after_comma_before_open_paren() {
    let content = r#"foo(a,(b))/*1*/
foo(a,(<b>c).d)/*2*/"#;
    let mut s = Session::new_for_test("formattingSpaceAfterCommaBeforeOpenParen", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"foo(a, (b));"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"foo(a, (<b>c).d);"#);
}
