use tsox_lsp::fourslash::{self, Session};

#[ignore = "needs live LSP session"]
#[test]
fn formatting_on_do_while_no_semicolon() {
    let content = r#"/*2*/do {
/*3*/    for (var i = 0; i < 10; i++)
/*4*/        i -= 2
/*5*/        }/*1*/while (1 !== 1)"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::verify_current_line_content(&mut s, r#"while (1 !== 1)"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"do {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    for (var i = 0; i < 10; i++)"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"        i -= 2"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
