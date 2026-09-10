use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_comments_before_errors() {
    let content = r#"namespace A {
    interface B {
        // a
        // b
        baz();
/*0*/        // d /*1*/asd a
        // e
        foo();
        // f asd
        // g as
        bar();
    }
}"#;
    let mut s = Session::new_for_test("formattingCommentsBeforeErrors", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_marker(&mut s, "0");
    fourslash::verify_current_line_content(&mut s, r#"        // d "#);
}
