use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_in_destructuring1() {
    let content = r#"interface let { }
/*1*/var x: let         [];

function foo() {
    'use strict'
/*2*/    let        [x] = [];
/*3*/    const      [x] = [];
/*4*/    for (let[x] = [];x < 1;) {
    }
}"#;
    let mut s = Session::new_for_test("formattingInDestructuring1", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var x: let[];"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    let [x] = [];"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    const [x] = [];"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    for (let [x] = []; x < 1;) {"#);
    // TODO: }
}
