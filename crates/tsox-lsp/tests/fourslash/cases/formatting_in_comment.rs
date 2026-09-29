use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_in_comment() {
    let content = r#"class A {
foo(              ); // /*1*/
}
function foo() {       var x;       } // /*2*/"#;
    let mut s = Session::new_for_test("formattingInComment", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"foo(              ); // ;"#);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.Insert(t, "
}
