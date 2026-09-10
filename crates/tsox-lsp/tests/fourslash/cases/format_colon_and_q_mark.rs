use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_colon_and_q_mark() {
    let content = r#"class foo {/*1*/
    constructor (n?: number, m = 5, o?: string) { }/*2*/
    x:number = 1?2:3;/*3*/
}/*4*/"#;
    let mut s = Session::new_for_test("formatColonAndQMark", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"class foo {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(n?: number, m = 5, o?: string) { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    x: number = 1 ? 2 : 3;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
