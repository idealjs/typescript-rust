use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_nested_do_while_by_enter() {
    let content = r#"/*2*/do{
/*3*/do/*1*/{
/*4*/do{
/*5*/}while(a!==b)
/*6*/}while(a!==b)
/*7*/}while(a!==b)"#;
    let mut s = Session::new_for_test("formattingOnNestedDoWhileByEnter", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::verify_current_line_content(&mut s, r#"    {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"do{"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    do"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"do{"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"}while(a!==b)"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"}while(a!==b)"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"}while(a!==b)"#);
}
