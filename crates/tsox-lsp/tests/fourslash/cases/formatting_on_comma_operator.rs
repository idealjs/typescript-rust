use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_comma_operator() {
    let content = r#"var v1 = ((1, 2, 3), 4, 5, (6, 7));/*1*/
function f1() {
    var a = 1;
    return a, v1, a;/*2*/
}"#;
    let mut s = Session::new_for_test("formattingOnCommaOperator", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var v1 = ((1, 2, 3), 4, 5, (6, 7));"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    return a, v1, a;"#);
}
