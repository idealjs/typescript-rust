use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_if_in_else_block() {
    let content = r#"if (true) {
}
else {
    if (true) {
        /*1*/
}"#;
    let mut s = Session::new_for_test("formattingIfInElseBlock", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "}");
    // TODO: f.VerifyCurrentLineContent(t, `
}
