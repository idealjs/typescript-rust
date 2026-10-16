use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_q_mark() {
    let content = r#"interface A {
/*1*/    foo?     ();
/*2*/    foo?             <T>();
}"#;
    let mut s = Session::new_for_test("formattingQMark", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    foo?();"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    foo?<T>();"#);
}
